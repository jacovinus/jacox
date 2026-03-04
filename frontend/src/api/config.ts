import client from './client';

export interface ProviderInfo {
    id: string;
    active: boolean;
    supported_models: string[];
    status: 'online' | 'offline' | 'unverified';
}

export interface ActiveProviderDetail {
    id: string;
    status: 'online' | 'offline';
    supported_models: string[];
    active_model: string | null;
    error?: string;
}

export const configApi = {
    listProviders: async (): Promise<ProviderInfo[]> => {
        const resp = await client.get('/config/providers');
        return resp.data;
    },
    setActiveProvider: async (providerId: string): Promise<void> => {
        await client.post('/config/active-provider', { provider_id: providerId });
    },
    getActiveProviderInfo: async (): Promise<ActiveProviderDetail> => {
        const resp = await client.get('/config/active-provider');
        return resp.data;
    },
    verifyActiveProvider: async (): Promise<{ status: string, error?: string }> => {
        const resp = await client.post('/config/active-provider/verify');
        return resp.data;
    },
    getActiveModel: async (): Promise<{ model_id: string | null }> => {
        const resp = await client.get('/config/active-model');
        return resp.data;
    },
    setActiveModel: async (modelId: string | null): Promise<void> => {
        await client.post('/config/active-model', { model_id: modelId });
    }
};
