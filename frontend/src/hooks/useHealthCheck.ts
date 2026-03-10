import { useState, useEffect } from 'react';
import api from '../api/client';

const HEALTH_CHECK_URL = '/health';

export interface HealthStatus {
    isOnline: boolean;
    apiConnected: boolean;
    dbConnected: boolean;
    isRetrying: boolean;
}

export const useHealthCheck = (): HealthStatus => {
    const [status, setStatus] = useState<HealthStatus>({
        isOnline: true,
        apiConnected: true,
        dbConnected: true,
        isRetrying: false,
    });

    useEffect(() => {
        const checkHealth = async () => {
            try {
                // Use the configured API client so the Authorization header is included
                const response = await api.get(HEALTH_CHECK_URL, { timeout: 2000 });
                const data = response.data;
                
                setStatus({
                    isOnline: true,
                    apiConnected: data.api === 'connected',
                    dbConnected: data.database === 'connected',
                    isRetrying: false,
                });
            } catch (error) {
                setStatus({
                    isOnline: false,
                    apiConnected: false,
                    dbConnected: false,
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
