import React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Zap, ChevronDown, Check, Loader2, Github, AlertCircle, RefreshCw } from 'lucide-react';
import { configApi } from '../api/config';
import { clsx } from 'clsx';

export const ProviderSelector = () => {
    const [isOpen, setIsOpen] = React.useState(false);
    const queryClient = useQueryClient();

    const { data: providers, isLoading } = useQuery({
        queryKey: ['providers'],
        queryFn: () => configApi.listProviders()
    });

    const { data: activeDetail, isLoading: isLoadingDetail } = useQuery({
        queryKey: ['active-provider-detail'],
        queryFn: () => configApi.getActiveProviderInfo(),
        refetchInterval: 5000 // Match Chat.tsx for synchronization
    });

    const switchMutation = useMutation({
        mutationFn: (id: string) => configApi.setActiveProvider(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['providers'] });
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
            setIsOpen(false);
        }
    });

    const verifyMutation = useMutation({
        mutationFn: () => configApi.verifyActiveProvider(),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
        }
    });

    const selectModelMutation = useMutation({
        mutationFn: (id: string) => configApi.setActiveModel(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['active-provider-detail'] });
        }
    });

    if (isLoading) return <div className="p-4 flex justify-center"><Loader2 className="animate-spin text-monokai-aqua w-5 h-5" /></div>;

    const providerList = Array.isArray(providers) ? providers : [];
    const activeProvider = providerList.find(p => p.active);

    if (providerList.length === 0) {
        return (
            <div className="px-4 py-3 rounded-xl bg-monokai-red/10 border border-monokai-red/20 text-[10px] text-monokai-red font-mono">
                API Error: Providers not found. Restart backend.
            </div>
        );
    }

    return (
        <div className="relative">
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="w-full flex items-center gap-3 px-4 py-3 rounded-xl bg-gruv-dark-3 hover:bg-gruv-dark-2 border border-gruv-dark-4/30 transition-all group"
            >
                <div className="relative">
                    <div className="w-8 h-8 rounded-lg bg-monokai-aqua/10 flex items-center justify-center">
                        {activeProvider?.id === 'copilot' ? (
                            <Github className="w-4 h-4 text-monokai-purple" />
                        ) : (
                            <Zap className={clsx(
                                "w-4 h-4 transition-colors",
                                activeProvider?.id === 'ollama' ? "text-monokai-orange" :
                                    activeProvider?.id === 'openai' ? "text-monokai-green" : "text-monokai-aqua"
                            )} />
                        )}
                    </div>
                    {activeDetail && (
                        <div className={clsx(
                            "absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-gruv-dark-3",
                            activeDetail.status === 'online' ? "bg-monokai-green" : "bg-monokai-red"
                        )} />
                    )}
                </div>
                <div className="flex flex-col items-start overflow-hidden">
                    <span className="text-[10px] text-gruv-light-4 font-mono uppercase tracking-widest flex items-center gap-1">
                        Active Manager
                        {isLoadingDetail && <Loader2 className="w-2 h-2 animate-spin" />}
                    </span>
                    <span className="text-sm font-bold text-gruv-light-1 truncate capitalize">
                        {activeProvider?.id || 'Loading...'}
                    </span>
                </div>
                <ChevronDown className={clsx(
                    "ml-auto w-4 h-4 text-gruv-light-4 transition-transform duration-300",
                    isOpen && "rotate-180"
                )} />
            </button>

            {isOpen && (
                <>
                    <div
                        className="fixed inset-0 z-40"
                        onClick={() => setIsOpen(false)}
                    />
                    <div className="absolute top-full left-0 right-0 mt-2 z-50 glass rounded-2xl border border-gruv-dark-4/40 shadow-2xl overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200">
                        <div className="p-2 flex flex-col gap-1">
                            <div className="px-3 py-2 text-[10px] text-gruv-light-4 font-mono uppercase tracking-widest border-b border-gruv-dark-4/20 mb-1 flex justify-between items-center">
                                Providers
                                <button
                                    onClick={(e) => { e.stopPropagation(); verifyMutation.mutate(); }}
                                    className="hover:text-monokai-aqua transition-colors"
                                    title="Verify Connection"
                                >
                                    <RefreshCw className={clsx("w-3 h-3", verifyMutation.isPending && "animate-spin")} />
                                </button>
                            </div>
                            {providerList.map(p => (
                                <button
                                    key={p.id}
                                    onClick={() => switchMutation.mutate(p.id)}
                                    disabled={p.active || switchMutation.isPending}
                                    className={clsx(
                                        "w-full flex items-center justify-between p-3 rounded-xl transition-all",
                                        p.active
                                            ? "bg-monokai-aqua/10 text-monokai-aqua"
                                            : "hover:bg-gruv-dark-3 text-gruv-light-4 hover:text-gruv-light-1"
                                    )}
                                >
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-semibold capitalize">{p.id}</span>
                                        {p.active && activeDetail?.status === 'offline' && (
                                            <span title="Connection Error">
                                                <AlertCircle className="w-3 h-3 text-monokai-red" />
                                            </span>
                                        )}
                                    </div>
                                    {p.active && <Check className="w-4 h-4" />}
                                    {switchMutation.isPending && switchMutation.variables === p.id && (
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                    )}
                                </button>
                            ))}

                            {activeDetail && activeDetail.supported_models.length > 0 && (
                                <div className="mt-2 pt-2 border-t border-gruv-dark-4/20">
                                    <div className="px-3 py-2 text-[10px] text-gruv-light-4 font-mono uppercase tracking-widest mb-1">
                                        Available Models
                                    </div>
                                    <div className="px-2 max-h-48 overflow-y-auto scrollbar-thin">
                                        {activeDetail.supported_models.map(m => (
                                            <button
                                                key={m}
                                                onClick={(e) => { e.stopPropagation(); selectModelMutation.mutate(m); }}
                                                className={clsx(
                                                    "w-full px-2 py-1.5 text-[11px] flex items-center justify-between rounded-lg transition-colors",
                                                    activeDetail.active_model === m || (!activeDetail.active_model && m === activeDetail.supported_models[0]) // Fallback check or just if matches
                                                        ? "text-monokai-aqua bg-monokai-aqua/5 font-bold"
                                                        : "text-gruv-light-3 hover:text-gruv-light-1 hover:bg-gruv-dark-3"
                                                )}
                                            >
                                                <div className="flex items-center gap-2">
                                                    <div className={clsx(
                                                        "w-1 h-1 rounded-full",
                                                        activeDetail.active_model === m ? "bg-monokai-aqua" : "bg-gruv-dark-4"
                                                    )} />
                                                    {m}
                                                </div>
                                                {activeDetail.active_model === m && <Check className="w-3 h-3" />}
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </>
            )}
        </div>
    );
};
