// Extension: commit-and-push
// Commit and push after structural changes, checking docs are up to date first.
// Provides a tool the agent calls after making structural changes (new files,
// renamed modules, changed CLI interface, etc.). The tool:
//   1. Asks the agent to verify local markdown docs (README.md,
//      copilot-instructions.md) are consistent with the changes.
//   2. Stages, commits, and pushes.

import { execFile } from "node:child_process";
import { joinSession } from "@github/copilot-sdk/extension";

const DOCS_TO_CHECK = [
    "README.md",
    ".github/copilot-instructions.md",
];

function git(...args) {
    return new Promise((resolve) => {
        execFile("git", ["--no-pager", ...args], { cwd: process.cwd() }, (err, stdout, stderr) => {
            if (err) resolve({ ok: false, output: (stderr || err.message).trim() });
            else resolve({ ok: true, output: stdout.trim() });
        });
    });
}

const session = await joinSession({
    tools: [
        {
            name: "commit_and_push",
            description:
                "Commit and push the current changes. " +
                "IMPORTANT: Before calling this tool you MUST first check that local markdown " +
                "documentation (README.md, .github/copilot-instructions.md) is still accurate " +
                "after the structural changes you made. If any doc is stale, update it first, " +
                "then call this tool. Pass a short conventional-commit message and an optional " +
                "body describing what changed.",
            parameters: {
                type: "object",
                properties: {
                    message: {
                        type: "string",
                        description: "Short commit subject line (conventional-commit style)",
                    },
                    body: {
                        type: "string",
                        description: "Optional longer commit body",
                    },
                },
                required: ["message"],
            },
            handler: async (args) => {
                const lines = [];

                // 1. Check which tracked docs exist and show their status
                const { output: diffNames } = await git("diff", "--name-only");
                const { output: diffCached } = await git("diff", "--cached", "--name-only");
                const changedFiles = new Set(
                    [...diffNames.split("\n"), ...diffCached.split("\n")].filter(Boolean)
                );

                const staleDocs = [];
                for (const doc of DOCS_TO_CHECK) {
                    if (!changedFiles.has(doc)) {
                        // Doc wasn't touched — flag it for awareness
                        staleDocs.push(doc);
                    }
                }

                if (staleDocs.length > 0) {
                    lines.push(
                        `Note: The following docs were NOT updated in this changeset: ${staleDocs.join(", ")}. ` +
                        `If they need updating, abort and update them first.`
                    );
                }

                // 2. Stage all changes
                const stage = await git("add", "-A");
                if (!stage.ok) return `Failed to stage: ${stage.output}`;

                // 3. Check there's something to commit
                const status = await git("status", "--porcelain");
                if (!status.output) return "Nothing to commit — working tree clean.";

                // 4. Build commit message
                let commitMsg = args.message;
                const trailer = "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>";
                if (args.body) {
                    commitMsg += "\n\n" + args.body + "\n\n" + trailer;
                } else {
                    commitMsg += "\n\n" + trailer;
                }

                // 5. Commit
                const commit = await git("commit", "-m", commitMsg);
                if (!commit.ok) return `Commit failed: ${commit.output}`;
                lines.push(commit.output);

                // 6. Push
                const push = await git("push");
                if (!push.ok) {
                    // Try with --set-upstream
                    const branch = await git("branch", "--show-current");
                    const pushUp = await git("push", "--set-upstream", "origin", branch.output);
                    if (!pushUp.ok) return `Push failed: ${pushUp.output}`;
                    lines.push(pushUp.output || "Pushed (set upstream).");
                } else {
                    lines.push(push.output || "Pushed.");
                }

                return lines.join("\n");
            },
        },
    ],
    hooks: {
        onPostToolUse: async (input) => {
            // After the agent creates or edits files, remind it about this tool
            // if the change looks structural (new .rs file, deleted file, changed Cargo.toml, changed cli.rs)
            if (input.toolName !== "create" && input.toolName !== "edit") return;

            const path = String(input.toolArgs?.path || "");
            const isStructural =
                path.endsWith("Cargo.toml") ||
                path.endsWith("cli.rs") ||
                path.endsWith("main.rs") ||
                (input.toolName === "create" && path.endsWith(".rs"));

            if (isStructural) {
                return {
                    additionalContext:
                        "You just made a structural change. When you are done with all related changes, " +
                        "review README.md and .github/copilot-instructions.md for accuracy, update them " +
                        "if needed, then use the commit_and_push tool to commit and push everything.",
                };
            }
        },
    },
});
