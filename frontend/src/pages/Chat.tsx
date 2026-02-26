import { useState, useEffect, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Send, Plus, Trash2, Bot, User, Loader2, Edit3, Settings, X, Check, Code, Eye } from 'lucide-react';
import { sessionsApi } from '../api/sessions';
import { useChatStream } from '../hooks/useChatStream';
import { clsx } from 'clsx';
import { MarkdownContent } from '../components/MarkdownContent';

export const Chat = () => {
    const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
    const [input, setInput] = useState('');
    const [isEditingName, setIsEditingName] = useState(false);
    const [editingNameValue, setEditingNameValue] = useState('');
    const [isMetadataModalOpen, setIsMetadataModalOpen] = useState(false);
    const [metadataValue, setMetadataValue] = useState('');
    const [rawMessageIndices, setRawMessageIndices] = useState<Set<number>>(new Set());

    const messagesEndRef = useRef<HTMLDivElement>(null);
    const queryClient = useQueryClient();

    // ... existing queries ...
    const { data: sessions, isLoading: sessionsLoading } = useQuery({
        queryKey: ['sessions'],
        queryFn: () => sessionsApi.list()
    });

    const activeSession = sessions?.find(s => s.id === activeSessionId);

    const { data: messageHistory } = useQuery({
        queryKey: ['messages', activeSessionId],
        queryFn: () => sessionsApi.getMessages(activeSessionId!),
        enabled: !!activeSessionId
    });

    // Stream Hook
    const {
        messages,
        setMessages,
        isStreaming,
        error: chatError,
        connect,
        sendMessage
    } = useChatStream(activeSessionId);

    // Auto-connect when session ID changes
    useEffect(() => {
        if (activeSessionId) {
            const apiKey = localStorage.getItem('jacox_api_key') || 'sk-dev-key-123';
            connect(apiKey);
        }
    }, [activeSessionId, connect]);

    // Sync history to stream hook
    useEffect(() => {
        if (messageHistory) {
            setMessages(messageHistory);
        }
    }, [messageHistory, setMessages]);

    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages]);

    // Mutations
    const updateSession = useMutation({
        mutationFn: ({ id, name, metadata }: { id: string, name?: string, metadata?: any }) =>
            sessionsApi.update(id, { name, metadata }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['sessions'] });
            setIsEditingName(false);
            setIsMetadataModalOpen(false);
        }
    });

    const createSession = useMutation({
        // ...
        mutationFn: (name: string) => sessionsApi.create({ name }),
        onSuccess: (newSession) => {
            queryClient.invalidateQueries({ queryKey: ['sessions'] });
            setActiveSessionId(newSession.id);
            const apiKey = localStorage.getItem('jacox_api_key') || 'sk-dev-key-123';
            connect(apiKey);
        }
    });

    const deleteSession = useMutation({
        mutationFn: (id: string) => sessionsApi.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['sessions'] });
            if (activeSessionId) setActiveSessionId(null);
        }
    });

    const handleSend = () => {
        if (!input.trim() || isStreaming) return;
        sendMessage(input);
        setInput('');
    };

    const handleSessionChange = (id: string) => {
        setActiveSessionId(id);
        const apiKey = localStorage.getItem('jacox_api_key') || 'sk-dev-key-123';
        connect(apiKey);
    };

    return (
        <div className="flex h-[calc(100vh-140px)] gap-6">
            {/* Session List Sidebar (Local to Chat) */}
            <div className="w-64 shrink-0 glass rounded-2xl flex flex-col overflow-hidden">
                <div className="p-4 border-b border-gruv-dark-4/20">
                    <button
                        onClick={() => createSession.mutate(`New Session ${sessions?.length || 0 + 1}`)}
                        className="w-full btn-primary flex items-center justify-center gap-2 py-2 text-sm"
                    >
                        <Plus className="w-4 h-4" />
                        New Chat
                    </button>
                </div>
                <div className="flex-grow overflow-y-auto p-2 flex flex-col gap-1">
                    {sessionsLoading ? (
                        <div className="flex justify-center p-8"><Loader2 className="animate-spin text-monokai-aqua" /></div>
                    ) : (
                        sessions?.map(s => (
                            <div
                                key={s.id}
                                onClick={() => handleSessionChange(s.id)}
                                className={clsx(
                                    "p-3 rounded-xl cursor-pointer transition-all duration-200 group flex items-center justify-between",
                                    activeSessionId === s.id ? "bg-gruv-dark-3 text-monokai-aqua" : "hover:bg-gruv-dark-3/50 text-gruv-light-4"
                                )}
                            >
                                <span className="truncate text-sm font-semibold">{s.name}</span>
                                <Trash2
                                    className="w-4 h-4 opacity-0 group-hover:opacity-100 hover:text-monokai-pink transition-all"
                                    onClick={(e) => { e.stopPropagation(); deleteSession.mutate(s.id); }}
                                />
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Main Chat Area */}
            <div className="flex-grow min-w-0 glass rounded-2xl flex flex-col overflow-hidden">
                {!activeSessionId ? (
                    <div className="flex-grow flex flex-col items-center justify-center text-gruv-light-4 p-8 text-center">
                        <Bot className="w-16 h-16 mb-4 opacity-20" />
                        <h2 className="text-xl font-bold text-gruv-light-2">Select a session to start</h2>
                        <p className="max-w-xs mt-2">Choose an existing conversation from the left or create a new one to chat with your local models.</p>
                    </div>
                ) : (
                    <>
                        <div className="p-4 border-b border-gruv-dark-4/20 flex justify-between items-center bg-gruv-dark-2/30">
                            <div className="flex items-center gap-3">
                                {isEditingName ? (
                                    <div className="flex items-center gap-2">
                                        <input
                                            type="text"
                                            value={editingNameValue}
                                            onChange={(e) => setEditingNameValue(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === 'Enter') updateSession.mutate({ id: activeSessionId!, name: editingNameValue });
                                                if (e.key === 'Escape') setIsEditingName(false);
                                            }}
                                            className="bg-gruv-dark-3 border border-monokai-aqua/50 rounded px-2 py-1 text-sm text-monokai-aqua outline-none w-64"
                                            autoFocus
                                        />
                                        <button onClick={() => updateSession.mutate({ id: activeSessionId!, name: editingNameValue })} className="text-monokai-green hover:scale-110 transition-transform"><Check className="w-4 h-4" /></button>
                                        <button onClick={() => setIsEditingName(false)} className="text-monokai-red hover:scale-110 transition-transform"><X className="w-4 h-4" /></button>
                                    </div>
                                ) : (
                                    <h2
                                        className="font-bold flex items-center gap-2 group cursor-pointer"
                                        onClick={() => {
                                            setIsEditingName(true);
                                            setEditingNameValue(activeSession?.name || '');
                                        }}
                                    >
                                        {activeSession?.name}
                                        <Edit3 className="w-3 h-3 opacity-0 group-hover:opacity-50 transition-opacity" />
                                    </h2>
                                )}
                                <div className="flex flex-col">
                                    {chatError ? (
                                        <span className="text-[10px] text-monokai-red font-mono">Error: {chatError}</span>
                                    ) : (
                                        <span className="text-[10px] text-monokai-green font-mono uppercase tracking-wider opacity-60">Connected</span>
                                    )}
                                </div>
                            </div>
                            <button
                                onClick={() => {
                                    setIsMetadataModalOpen(true);
                                    setMetadataValue(JSON.stringify(activeSession?.metadata || {}, null, 2));
                                }}
                                className="p-2 hover:bg-gruv-dark-3 rounded-xl transition-colors text-gruv-light-4 hover:text-monokai-aqua flex items-center gap-2 text-xs"
                            >
                                <Settings className="w-4 h-4" />
                                Metadata
                            </button>
                        </div>

                        <div className="flex-grow overflow-y-auto p-6 flex flex-col gap-6">
                            {messages.map((m, i) => (
                                <div
                                    key={i}
                                    className={clsx(
                                        "flex gap-4 max-w-[85%]",
                                        m.role === 'user' ? "self-end flex-row-reverse" : "self-start"
                                    )}
                                >
                                    <div className={clsx(
                                        "w-10 h-10 rounded-xl flex items-center justify-center shrink-0 shadow-lg",
                                        m.role === 'user' ? "bg-gruv-dark-3" : "bg-monokai-pink/10 border border-monokai-pink/20"
                                    )}>
                                        {m.role === 'user' ? <User className="w-5 h-5" /> : <Bot className="w-5 h-5 text-monokai-pink" />}
                                    </div>
                                    <div className={clsx(
                                        "p-4 rounded-2xl text-sm leading-relaxed relative group/message",
                                        m.role === 'user' ? "bg-gruv-dark-3 text-gruv-light-1" : "bg-gruv-dark-2/50 border border-gruv-dark-4/30"
                                    )}>
                                        <button
                                            onClick={() => {
                                                const next = new Set(rawMessageIndices);
                                                if (next.has(i)) next.delete(i);
                                                else next.add(i);
                                                setRawMessageIndices(next);
                                            }}
                                            className="absolute -top-3 right-0 opacity-0 group-hover/message:opacity-100 transition-opacity p-1.5 bg-gruv-dark-4 border border-gruv-dark-4/50 rounded-lg shadow-xl text-gruv-light-4 hover:text-monokai-aqua z-10"
                                            title={rawMessageIndices.has(i) ? "Show Rendered" : "Show Raw"}
                                        >
                                            {rawMessageIndices.has(i) ? <Eye className="w-3 h-3" /> : <Code className="w-3 h-3" />}
                                        </button>

                                        {rawMessageIndices.has(i) ? (
                                            <pre className="whitespace-pre-wrap font-mono text-[0.85em] text-gruv-light-3">
                                                {m.content}
                                            </pre>
                                        ) : (
                                            <MarkdownContent content={m.content} />
                                        )}
                                    </div>
                                </div>
                            ))}
                            <div ref={messagesEndRef} />
                        </div>

                        <div className="p-6 bg-gruv-dark-0/50 border-t border-gruv-dark-4/20">
                            <div className="relative">
                                <textarea
                                    value={input}
                                    onChange={(e) => setInput(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && (e.preventDefault(), handleSend())}
                                    placeholder="Type a message... (Enter to send, Shift+Enter for new line)"
                                    className="w-full bg-gruv-dark-3 border border-gruv-dark-4 text-gruv-light-1 rounded-2xl py-4 pl-4 pr-16 focus:outline-none focus:border-monokai-pink transition-colors resize-none h-16 min-h-[64px]"
                                />
                                <button
                                    onClick={handleSend}
                                    disabled={!input.trim() || isStreaming}
                                    className="absolute right-3 top-1/2 -translate-y-1/2 p-2 bg-monokai-pink text-white rounded-xl disabled:opacity-50 disabled:cursor-not-allowed hover:scale-105 transition-all shadow-lg"
                                >
                                    <Send className="w-5 h-5" />
                                </button>
                            </div>
                        </div>
                    </>
                )}
            </div>

            {/* Metadata Editor Modal */}
            {isMetadataModalOpen && (
                <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                    <div className="w-full max-w-2xl glass rounded-3xl overflow-hidden shadow-2xl border border-gruv-dark-4/30">
                        <div className="p-6 border-b border-gruv-dark-4/20 flex justify-between items-center bg-gruv-dark-2/30">
                            <h3 className="font-bold flex items-center gap-2">
                                <Settings className="w-5 h-5 text-monokai-aqua" />
                                Session Metadata
                            </h3>
                            <button onClick={() => setIsMetadataModalOpen(false)} className="p-1 hover:bg-gruv-dark-4 rounded-lg transition-colors">
                                <X className="w-5 h-5" />
                            </button>
                        </div>
                        <div className="p-6">
                            <p className="text-xs text-gruv-light-4 mb-4">Edit raw JSON metadata for this conversation. Useful for setting context or tags.</p>
                            <textarea
                                value={metadataValue}
                                onChange={(e) => setMetadataValue(e.target.value)}
                                className="w-full h-96 bg-gruv-dark-3 border border-gruv-dark-4 rounded-2xl p-4 font-mono text-xs text-monokai-aqua leading-relaxed focus:outline-none focus:border-monokai-aqua transition-colors resize-none"
                                spellCheck={false}
                            />
                        </div>
                        <div className="p-6 border-t border-gruv-dark-4/20 flex justify-end gap-3 bg-gruv-dark-2/30">
                            <button
                                onClick={() => setIsMetadataModalOpen(false)}
                                className="px-5 py-2 hover:bg-gruv-dark-4 rounded-xl transition-colors text-sm font-semibold"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={() => {
                                    try {
                                        const metadata = JSON.parse(metadataValue);
                                        updateSession.mutate({ id: activeSessionId!, metadata });
                                    } catch (e) {
                                        alert('Invalid JSON format');
                                    }
                                }}
                                className="btn-primary px-5 py-2 text-sm"
                            >
                                Save Changes
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
