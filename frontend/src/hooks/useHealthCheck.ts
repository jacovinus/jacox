import { useState, useEffect } from 'react';
import axios from 'axios';

const HEALTH_CHECK_URL = import.meta.env.VITE_API_BASE_URL + '/health';

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
                const response = await axios.get(HEALTH_CHECK_URL, { timeout: 2000 });
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
