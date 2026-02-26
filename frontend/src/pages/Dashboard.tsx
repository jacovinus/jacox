import { useQuery } from '@tanstack/react-query';
import { sessionsApi } from '../api/sessions';
import {
    Activity,
    Clock,
    MessageCircle,
    Layers,
    Zap,
    HardDrive,
    Loader2
} from 'lucide-react';

const StatCard = ({ title, value, icon: Icon, trend, color, isLoading }: any) => (
    <div className="glass p-6 rounded-2xl flex flex-col gap-4 hover:border-gruv-light-4/50 transition-colors">
        <div className="flex justify-between items-start">
            <div className={`p-3 rounded-xl bg-${color}/10 text-${color}`}>
                <Icon className="w-6 h-6" />
            </div>
            {trend && (
                <span className="text-xs font-mono text-monokai-green">+{trend}%</span>
            )}
        </div>
        <div>
            <p className="text-gruv-light-4 text-sm font-semibold uppercase tracking-wider">{title}</p>
            {isLoading ? (
                <div className="h-9 flex items-center"><Loader2 className="w-5 h-5 animate-spin text-gruv-light-4" /></div>
            ) : (
                <h3 className="text-3xl font-bold mt-1">{value}</h3>
            )}
        </div>
    </div>
);

import { useHealthCheck } from '../hooks/useHealthCheck';

import { AlertCircle, AlertTriangle } from 'lucide-react';
import { Tooltip } from '../components/Tooltip';
import { cn } from '../utils/cn';

export const Dashboard = () => {
    const health = useHealthCheck();

    const { data: stats, isLoading: statsLoading } = useQuery({
        queryKey: ['system-stats'],
        queryFn: () => sessionsApi.getStats(),
        refetchInterval: 5000
    });

    const { data: sessions, isLoading: sessionsLoading } = useQuery({
        queryKey: ['sessions'],
        queryFn: () => sessionsApi.list(),
        retry: false
    });

    const formatBytes = (bytes: number) => {
        if (!bytes) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    };

    const formatNumber = (num: number) => {
        if (!num) return '0';
        if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
        if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
        return num.toLocaleString();
    };

    const safeSessions = Array.isArray(sessions) ? sessions : [];

    return (
        <div className="flex flex-col gap-8">
            <header className="flex justify-between items-start">
                {/* ... existing header content ... */}
                <div>
                    <h1 className="text-4xl font-bold">System Overview</h1>
                    <p className="text-gruv-light-4">Real-time performance metrics and usage data.</p>
                </div>
                <div className="flex gap-3">
                    <Tooltip content={health.apiConnected ? "API is responding correctly" : "Critical: API unreachable"}>
                        <div className={cn(
                            "px-3 py-1.5 rounded-full border flex items-center gap-2 font-mono text-xs transition-all",
                            health.apiConnected
                                ? "border-monokai-green/30 text-monokai-green bg-monokai-green/5"
                                : "border-monokai-red/50 text-monokai-red bg-monokai-red/10 shadow-[0_0_15px_rgba(249,38,114,0.2)]"
                        )}>
                            {!health.apiConnected && <AlertCircle className="w-3.5 h-3.5 animate-pulse" />}
                            <div className={cn("w-1.5 h-1.5 rounded-full", health.apiConnected ? "bg-monokai-green" : "bg-monokai-red")} />
                            API: {health.apiConnected ? 'READY' : 'DISCONNECTED'}
                        </div>
                    </Tooltip>

                    <Tooltip content={health.dbConnected ? "DuckDB connection active" : "Warning: Database disconnected"}>
                        <div className={cn(
                            "px-3 py-1.5 rounded-full border flex items-center gap-2 font-mono text-xs transition-all",
                            health.dbConnected
                                ? "border-monokai-aqua/30 text-monokai-aqua bg-monokai-aqua/5"
                                : "border-monokai-orange/50 text-monokai-orange bg-monokai-orange/10 shadow-[0_0_15px_rgba(253,151,31,0.2)]"
                        )}>
                            {!health.dbConnected && <AlertTriangle className="w-3.5 h-3.5 animate-pulse" />}
                            <div className={cn("w-1.5 h-1.5 rounded-full", health.dbConnected ? "bg-monokai-aqua" : "bg-monokai-orange")} />
                            DB: {health.dbConnected ? 'CONNECTED' : 'OFFLINE'}
                        </div>
                    </Tooltip>
                </div>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatCard
                    title="Total Sessions"
                    value={formatNumber(stats?.total_sessions || 0)}
                    icon={Layers}
                    color="monokai-aqua"
                    isLoading={statsLoading}
                />
                <StatCard
                    title="Total Messages"
                    value={formatNumber(stats?.total_messages || 0)}
                    icon={MessageCircle}
                    color="monokai-pink"
                    isLoading={statsLoading}
                />
                <StatCard
                    title="Tokens Used"
                    value={formatNumber(stats?.total_tokens || 0)}
                    icon={Zap}
                    color="monokai-purple"
                    isLoading={statsLoading}
                />
                <StatCard
                    title="DB Storage"
                    value={formatBytes(stats?.db_size_bytes || 0)}
                    icon={HardDrive}
                    color="gruv-yellow"
                    isLoading={statsLoading}
                />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div className="glass p-8 rounded-2xl h-80 flex flex-col items-center justify-center text-center">
                    <Activity className="text-monokai-aqua w-12 h-12 mb-4 opacity-40 animate-pulse" />
                    <h3 className="text-xl font-bold mb-2">System Active</h3>
                    <p className="text-sm text-gruv-light-4 max-w-xs">WebSocket is listening for events on the default port.</p>
                </div>

                <div className="glass p-8 rounded-2xl h-80 overflow-hidden flex flex-col">
                    <h3 className="text-xl font-bold mb-6 flex items-center gap-2">
                        <Clock className="text-monokai-purple w-5 h-5" />
                        Recent Sessions
                    </h3>
                    <div className="flex flex-col gap-4 overflow-y-auto">
                        {sessionsLoading ? (
                            <div className="flex justify-center p-8"><Loader2 className="animate-spin text-monokai-purple" /></div>
                        ) : safeSessions.length > 0 ? (
                            safeSessions.slice(0, 5).map((s) => (
                                <a
                                    key={s.id}
                                    href={`/chat?session=${s.id}`}
                                    className="flex items-center gap-4 p-3 hover:bg-gruv-dark-4/20 rounded-xl transition-colors cursor-pointer"
                                    onClick={(e) => {
                                        e.preventDefault();
                                        window.location.href = `/chat?session=${s.id}`;
                                    }}
                                >
                                    <div className="w-10 h-10 rounded-full bg-gruv-dark-4 flex items-center justify-center">
                                        <MessageCircle className="w-5 h-5 text-gruv-light-1" />
                                    </div>
                                    <div className="flex-grow min-w-0">
                                        <p className="font-semibold text-sm truncate">{s.name}</p>
                                        <p className="text-xs text-gruv-light-4">{new Date(s.created_at).toLocaleDateString()} • {s.metadata?.message_count || 0} messages</p>
                                    </div>
                                </a>
                            ))
                        ) : (
                            <div className="flex-grow flex flex-col items-center justify-center text-gruv-light-4 opacity-50">
                                <MessageCircle className="w-10 h-10 mb-2 opacity-20" />
                                <p className="text-sm font-mono">No recent activity</p>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};
