import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Settings as SettingsIcon, Save, Key, Cpu, Sparkles, Loader2, Check, AlertCircle, RefreshCw, Github, Zap } from 'lucide-react';
import { configApi } from '../api/config';
import { clsx } from 'clsx';

export const Settings = () => {
    const queryClient = useQueryClient();
    const [apiKey, setApiKey] = useState(localStorage.getItem('jacox_api_key') || 'sk-dev-key-123');

    const { data: providers, isLoading: isLoadingProviders } = useQuery({
        queryKey: ['providers'],
        queryFn: () => configApi.listProviders()
    });

    const { data: activeDetail, isLoading: isLoadingDetail } = useQuery({
        queryKey: ['active-provider-detail'],
        queryFn: () => configApi.getActiveProviderInfo(),
        refetchInterval: 30000
    });

    const switchProviderMutation = useMutation({
        mutationFn: (id: string) => configApi.setActiveProvider(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['providers'] });
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
        }
    });

    const selectModelMutation = useMutation({
        mutationFn: (id: string) => configApi.setActiveModel(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
        }
    });

    const verifyMutation = useMutation({
        mutationFn: () => configApi.verifyActiveProvider(),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
        }
    });

    const handleSaveAuth = () => {
        localStorage.setItem('jacox_api_key', apiKey);
        alert('Authentication settings saved locally!');
    };

    const activeProvider = providers?.find(p => p.active);

    return (
        <div className="flex flex-col gap-8 pb-12">
            <header>
                <h1 className="text-4xl font-bold flex items-center gap-3">
                    <SettingsIcon className="text-gruv-yellow" />
                    Settings
                </h1>
                <p className="text-gruv-light-4">Manage your Stepbit instance, LLM providers and model preferences.</p>
            </header>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                {/* Authentication Section */}
                <div className="glass p-8 rounded-2xl flex flex-col gap-6">
                    <h3 className="text-xl font-bold flex items-center gap-2">
                        <Key className="text-monokai-pink w-5 h-5" />
                        Platform Security
                    </h3>
                    <div className="flex flex-col gap-2">
                        <label className="text-xs font-mono text-gruv-light-4 uppercase">Stepbit API Key</label>
                        <input
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            className="bg-gruv-dark-3 border border-gruv-dark-4 rounded-xl py-3 px-4 focus:outline-none focus:border-monokai-pink transition-colors"
                        />
                        <p className="text-[10px] text-gruv-light-4 leading-relaxed mt-1">
                            This key is stored in your browser and used for local API access. Default development key is <code>sk-dev-key-123</code>.
                        </p>
                    </div>
                    <button onClick={handleSaveAuth} className="btn-primary w-full mt-2 flex items-center justify-center gap-2">
                        <Save className="w-4 h-4" />
                        Update Security Key
                    </button>
                </div>

                {/* Active Provider Status Section */}
                <div className="glass p-8 rounded-2xl flex flex-col gap-6">
                    <div className="flex justify-between items-center">
                        <h3 className="text-xl font-bold flex items-center gap-2">
                            <Cpu className="text-monokai-aqua w-5 h-5" />
                            LLM Infrastructure
                        </h3>
                        <button
                            onClick={() => verifyMutation.mutate()}
                            className="p-2 hover:bg-gruv-dark-3 rounded-lg transition-colors group"
                            title="Refresh Connection"
                        >
                            <RefreshCw className={clsx("w-4 h-4 text-gruv-light-4 group-hover:text-monokai-aqua", (verifyMutation.isPending || isLoadingDetail) && "animate-spin")} />
                        </button>
                    </div>

                    {isLoadingProviders || isLoadingDetail ? (
                        <div className="py-12 flex flex-col items-center gap-3">
                            <Loader2 className="w-8 h-8 animate-spin text-monokai-aqua" />
                            <span className="text-xs font-mono text-gruv-light-4">Synchronizing with Hub...</span>
                        </div>
                    ) : (
                        <div className="space-y-4">
                            <div className="p-5 bg-gruv-dark-3/50 rounded-2xl border border-gruv-dark-4/20 flex flex-col gap-4">
                                <div className="flex justify-between items-start">
                                    <div className="flex items-center gap-4">
                                        <div className="w-12 h-12 rounded-xl bg-monokai-aqua/10 flex items-center justify-center">
                                            {activeProvider?.id === 'copilot' ? (
                                                <Github className="w-6 h-6 text-monokai-purple" />
                                            ) : (activeProvider?.id === 'ollama' ? (
                                                <Zap className="w-6 h-6 text-monokai-orange" />
                                            ) : (activeProvider?.id === 'llmos' ? (
                                                <Cpu className="w-6 h-6 text-monokai-aqua" />
                                            ) : (
                                                <Sparkles className="w-6 h-6 text-monokai-aqua" />
                                            )))}
                                        </div>
                                        <div>
                                            <p className="text-xs text-gruv-light-4 font-mono uppercase">Active Manager</p>
                                            <p className="text-lg font-bold capitalize">{activeProvider?.id || 'Undefined'}</p>
                                        </div>
                                    </div>
                                    <div className={clsx(
                                        "px-3 py-1 rounded-full text-[10px] font-mono border uppercase flex items-center gap-2",
                                        activeDetail?.status === 'online'
                                            ? "bg-monokai-green/10 text-monokai-green border-monokai-green/20"
                                            : "bg-monokai-red/10 text-monokai-red border-monokai-red/20"
                                    )}>
                                        <div className={clsx("w-1.5 h-1.5 rounded-full animate-pulse", activeDetail?.status === 'online' ? "bg-monokai-green" : "bg-monokai-red")} />
                                        {activeDetail?.status || 'Offline'}
                                    </div>
                                </div>

                                {activeDetail?.error && (
                                    <div className="p-3 bg-monokai-red/5 border border-monokai-red/20 rounded-xl flex items-start gap-2 text-monokai-red text-[11px]">
                                        <AlertCircle className="w-4 h-4 shrink-0" />
                                        <span>Connection Error: {activeDetail.error}</span>
                                    </div>
                                )}
                            </div>

                            <div className="flex flex-col gap-2">
                                <label className="text-xs font-mono text-gruv-light-4 uppercase px-1">Active Model Portfolio</label>
                                <div className="grid grid-cols-1 gap-2 max-h-48 overflow-y-auto pr-2 scrollbar-thin">
                                    {(activeDetail?.supported_models || []).map(model => (
                                        <button
                                            key={model}
                                            onClick={() => selectModelMutation.mutate(model)}
                                            className={clsx(
                                                "flex items-center justify-between p-3 rounded-xl border transition-all text-left",
                                                activeDetail?.active_model === model
                                                    ? "bg-monokai-aqua/10 border-monokai-aqua/30 text-monokai-aqua ring-1 ring-monokai-aqua/20"
                                                    : "bg-gruv-dark-3/30 border-gruv-dark-4 hover:border-gruv-light-4/30 text-gruv-light-3"
                                            )}
                                        >
                                            <span className="text-sm font-medium">{model}</span>
                                            {activeDetail?.active_model === model && <Check className="w-4 h-4" />}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                {/* Provider Fleet Section */}
                <div className="lg:col-span-2 glass p-8 rounded-2xl flex flex-col gap-6">
                    <h3 className="text-xl font-bold flex items-center gap-2">
                        <Zap className="text-monokai-yellow w-5 h-5" />
                        Provider Fleet
                    </h3>
                    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
                        {providers?.map(provider => (
                            <button
                                key={provider.id}
                                onClick={() => switchProviderMutation.mutate(provider.id)}
                                disabled={provider.active || switchProviderMutation.isPending}
                                className={clsx(
                                    "p-6 rounded-2xl border transition-all flex flex-col gap-4 text-left group relative overflow-hidden",
                                    provider.active
                                        ? "bg-gruv-dark-2 border-monokai-aqua shadow-lg shadow-monokai-aqua/5"
                                        : "bg-gruv-dark-3/50 border-gruv-dark-4 hover:border-gruv-light-4/50 hover:bg-gruv-dark-2"
                                )}
                            >
                                <div className="flex justify-between items-start relative z-10">
                                    <div className={clsx(
                                        "w-10 h-10 rounded-xl flex items-center justify-center transition-colors",
                                        provider.active ? "bg-monokai-aqua/20 text-monokai-aqua" : "bg-gruv-dark-4 text-gruv-light-4 group-hover:text-gruv-light-1"
                                    )}>
                                        {provider.id === 'copilot' ? (
                                            <Github className="w-5 h-5" />
                                        ) : (provider.id === 'llmos' ? (
                                            <Cpu className="w-5 h-5" />
                                        ) : (
                                            <Zap className="w-5 h-5" />
                                        ))}
                                    </div>
                                    {provider.active && (
                                        <div className="w-6 h-6 rounded-full bg-monokai-aqua flex items-center justify-center">
                                            <Check className="w-3 h-3 text-gruv-dark-1" />
                                        </div>
                                    )}
                                </div>
                                <div className="relative z-10">
                                    <p className="font-bold text-lg capitalize">{provider.id}</p>
                                    <p className="text-xs text-gruv-light-4 font-mono mt-1">
                                        {provider.id === 'ollama' || provider.id === 'llmos' ? 'Local Compute' : 'Cloud Endpoint'}
                                    </p>
                                </div>
                                {provider.active && (
                                    <div className="absolute top-0 right-0 w-32 h-32 bg-monokai-aqua/5 blur-3xl -mr-16 -mt-16 rounded-full" />
                                )}
                            </button>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
};
