import { useState, useEffect, useCallback } from 'react';
import { pipelinesApi } from '../api/pipelines';
import type { LlmosStatus } from '../types';

export const useLlmos = (interval = 30000) => {
  const [status, setStatus] = useState<LlmosStatus>({ online: false, message: 'Checking...' });
  const [loading, setLoading] = useState(true);

  const checkStatus = useCallback(async () => {
    try {
      const currentStatus = await pipelinesApi.getLlmosStatus();
      setStatus(currentStatus);
    } catch (error) {
      setStatus({ online: false, message: 'Failed to reach backend' });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    checkStatus();
    if (interval > 0) {
      const timer = setInterval(checkStatus, interval);
      return () => clearInterval(timer);
    }
  }, [checkStatus, interval]);

  return { ...status, loading, refresh: checkStatus };
};
