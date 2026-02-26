import { AlertTriangle, RefreshCw } from 'lucide-react';

interface DisconnectedOverlayProps {
    isRetrying: boolean;
}

export const DisconnectedOverlay = ({ isRetrying }: DisconnectedOverlayProps) => {
    return (
        <div className="fixed inset-0 z-[100] bg-gruv-dark-0/80 backdrop-blur-md flex items-center justify-center p-6 text-center">
            <div className="glass p-12 rounded-3xl max-w-md border-monokai-orange/30 shadow-[0_0_50px_rgba(253,151,31,0.1)]">
                <div className="w-20 h-20 bg-monokai-orange/10 rounded-full flex items-center justify-center mx-auto mb-6">
                    <AlertTriangle className="w-10 h-10 text-monokai-orange animate-pulse" />
                </div>
                <h2 className="text-3xl font-bold mb-4 text-gruv-light-1">Connection Lost</h2>
                <p className="text-gruv-light-4 mb-8 leading-relaxed">
                    Unable to reach the Jacox server. Please ensure the backend is running and check your network connection.
                </p>
                <div className="flex items-center justify-center gap-3 text-monokai-aqua font-mono text-sm">
                    <RefreshCw className={isRetrying ? "w-4 h-4 animate-spin" : "w-4 h-4"} />
                    {isRetrying ? "Attempting to reconnect..." : "Waiting for connection..."}
                </div>
            </div>
        </div>
    );
};
