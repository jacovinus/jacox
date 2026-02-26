import React from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import {
    LayoutDashboard,
    MessageSquare,
    Database,
    Settings,
    ChevronRight,
    Cpu
} from 'lucide-react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

const SidebarItem = ({ to, icon: Icon, children }: { to: string, icon: any, children: React.ReactNode }) => (
    <NavLink
        to={to}
        className={({ isActive }) => cn(
            "flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 group",
            isActive
                ? "bg-monokai-pink text-white shadow-[0_0_15px_rgba(249,38,114,0.3)]"
                : "text-gruv-light-4 hover:bg-gruv-dark-3 hover:text-gruv-light-1"
        )}
    >
        <Icon className="w-5 h-5" />
        <span className="font-semibold">{children}</span>
        <ChevronRight className={cn(
            "ml-auto w-4 h-4 opacity-0 transition-all duration-300",
            "group-hover:opacity-100 group-hover:translate-x-1"
        )} />
    </NavLink>
);

import { useHealthCheck } from '../hooks/useHealthCheck';
import { DisconnectedOverlay } from './DisconnectedOverlay';

export const Layout = () => {
    const { isOnline, apiConnected, dbConnected, isRetrying } = useHealthCheck();

    return (
        <div className="flex h-screen bg-gruv-dark-1 text-gruv-light-1 overflow-hidden font-sans">
            {!isOnline && <DisconnectedOverlay isRetrying={isRetrying} />}
            {/* Sidebar */}
            <aside className="w-72 bg-gruv-dark-0 border-r border-gruv-dark-4/30 p-6 flex flex-col gap-8">
                <div className="flex items-center gap-3 px-2">
                    <div className="w-10 h-10 bg-gradient-to-br from-monokai-pink to-monokai-purple rounded-xl flex items-center justify-center shadow-lg">
                        <Cpu className="text-white w-6 h-6" />
                    </div>
                    <span className="text-2xl font-bold tracking-tight bg-gradient-to-r from-white to-gruv-light-4 bg-clip-text text-transparent">
                        Jacox
                    </span>
                </div>

                <nav className="flex flex-col gap-2 flex-grow">
                    <SidebarItem to="/" icon={LayoutDashboard}>Dashboard</SidebarItem>
                    <SidebarItem to="/chat" icon={MessageSquare}>Chat</SidebarItem>
                    <SidebarItem to="/database" icon={Database}>Database</SidebarItem>
                    <SidebarItem to="/settings" icon={Settings}>Settings</SidebarItem>
                </nav>

                <div className="mt-auto p-4 bg-gruv-dark-2/50 rounded-2xl border border-gruv-dark-4/20 space-y-3">
                    <div className="flex items-center gap-2">
                        <div className={cn(
                            "w-1.5 h-1.5 rounded-full animate-pulse",
                            isOnline && apiConnected ? "bg-monokai-green" : "bg-monokai-red"
                        )} />
                        <span className={cn(
                            "text-[10px] font-mono uppercase tracking-wider",
                            isOnline && apiConnected ? "text-monokai-green" : "text-monokai-red"
                        )}>
                            API: {isOnline && apiConnected ? "Online" : "Offline"}
                        </span>
                    </div>
                    <div className="flex items-center gap-2">
                        <div className={cn(
                            "w-1.5 h-1.5 rounded-full animate-pulse",
                            isOnline && dbConnected ? "bg-monokai-aqua" : "bg-monokai-orange"
                        )} />
                        <span className={cn(
                            "text-[10px] font-mono uppercase tracking-wider",
                            isOnline && dbConnected ? "text-monokai-aqua" : "text-monokai-orange"
                        )}>
                            DB: {isOnline && dbConnected ? "Online" : "Offline"}
                        </span>
                    </div>
                </div>
            </aside>

            {/* Main Content */}
            <main className="flex-grow overflow-auto relative">
                <div className="absolute top-0 left-0 w-full h-64 bg-gradient-to-b from-monokai-pink/5 to-transparent pointer-events-none" />
                <div className="p-8 relative z-10 w-full">
                    <Outlet />
                </div>
            </main>
        </div>
    );
};
