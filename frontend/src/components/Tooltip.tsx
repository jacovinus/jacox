import React, { useState } from 'react';
import { cn } from '../utils/cn'; // Assuming I can create a small utility for this or just use the one in Layout

export const Tooltip = ({ children, content, className }: { children: React.ReactNode, content: string, className?: string }) => {
    const [isVisible, setIsVisible] = useState(false);

    return (
        <div
            className="relative flex items-center"
            onMouseEnter={() => setIsVisible(true)}
            onMouseLeave={() => setIsVisible(false)}
        >
            {children}
            {isVisible && (
                <div className={cn(
                    "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-gruv-dark-0 border border-gruv-dark-4 text-gruv-light-1 text-[10px] font-mono rounded-lg shadow-2xl whitespace-nowrap z-[110] animate-in fade-in zoom-in duration-200",
                    className
                )}>
                    {content}
                    <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-gruv-dark-0" />
                </div>
            )}
        </div>
    );
};
