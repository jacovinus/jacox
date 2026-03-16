import { useState, useEffect } from 'react';
import api from '../api/client';

const HEALTH_CHECK_URL = 'health';

export interface HealthStatus {
    isOnline: boolean;
    apiConnected: boolean;
    dbConnected: boolean;
    llmosConnected: boolean;
    isRetrying: boolean;
}

export const useHealthCheck = (): HealthStatus => {
    const [status, setStatus] = useState<HealthStatus>({
        isOnline: true,
        apiConnected: true,
        dbConnected: true,
        llmosConnected: false,
        isRetrying: false,
    });

    useEffect(() => {
        const checkHealth = async () => {
            try {
                // Check general health
                const response = await api.get(HEALTH_CHECK_URL, { timeout: 2000 });
                const data = response.data;
                
                // Check stepbit-core specifically
                let llmosConnected = false;
                try {
                    const llmosRes = await api.get('stepbit-core/status', { timeout: 2000 });
                    llmosConnected = llmosRes.data.online;
                } catch (e) {
                    console.warn("stepbit-core heath check failed", e);
                }

                setStatus({
                    isOnline: true,
                    apiConnected: data.api === 'connected',
                    dbConnected: data.database === 'connected',
                    llmosConnected,
                    isRetrying: false,
                });
            } catch (error) {
                setStatus({
                    isOnline: false,
                    apiConnected: false,
                    dbConnected: false,
                    llmosConnected: false,
                    isRetrying: true,
                });
            }
        };

        checkHealth();
        const interval = setInterval(checkHealth, 5000);
        return () => clearInterval(interval);
    }, []);

    return status;
};
