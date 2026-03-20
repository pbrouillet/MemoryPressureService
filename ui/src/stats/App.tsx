import { useState, useEffect, useCallback } from "react";
import type { MemoryStats } from "@shared/types";
import { Card } from "@shared/Card";
import { Button } from "@shared/Button";
import { MemoryGauge } from "./MemoryGauge";
import { OverviewTable } from "./OverviewTable";
import { PageListTable } from "./PageListTable";
import { StandbyGrid } from "./StandbyGrid";
import "./App.css";

export function App() {
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [timestamp, setTimestamp] = useState<string>("Loading...");

  const handleUpdate = useCallback((data: MemoryStats) => {
    setError(null);
    setStats(data);
    setTimestamp("Updated " + new Date().toLocaleTimeString());
  }, []);

  const handleError = useCallback((msg: string) => {
    setError(msg);
  }, []);

  useEffect(() => {
    window.updateStats = handleUpdate;
    window.showError = handleError;
    return () => {
      window.updateStats = undefined;
      window.showError = undefined;
    };
  }, [handleUpdate, handleError]);

  const requestRefresh = () => {
    window.ipc.postMessage("refresh");
  };

  // Auto-refresh on mount
  useEffect(() => {
    requestRefresh();
  }, []);

  return (
    <>
      <h1>Memory Pressure Agent</h1>
      <p className="subtitle">{timestamp}</p>

      {error && <div className="error-msg">{error}</div>}

      <div className="toolbar">
        <Button primary onClick={requestRefresh}>
          ⟳ Refresh
        </Button>
      </div>

      {stats && (
        <>
          <Card title="Memory Load">
            <MemoryGauge stats={stats} />
          </Card>

          <Card title="System Overview">
            <OverviewTable stats={stats} />
          </Card>

          <Card title="Page Lists">
            <PageListTable stats={stats} />
          </Card>

          <Card title="Standby by Priority">
            <StandbyGrid stats={stats} />
          </Card>
        </>
      )}
    </>
  );
}
