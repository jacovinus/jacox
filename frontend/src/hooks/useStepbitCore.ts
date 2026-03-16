import { useState, useEffect, useCallback } from 'react';
import { pipelinesApi } from '../api/pipelines';
import type { StepbitCoreStatus } from '../types';

export const useStepbitCore = (interval = 30000) => {
  const [status, setStatus] = useState<StepbitCoreStatus>({ online: false, message: 'Checking...' });
  const [loading, setLoading] = useState(true);

  const checkStatus = useCallback(async () => {
    try {
      const currentStatus = await pipelinesApi.getStepbitCoreStatus();
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
