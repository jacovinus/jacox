import { useState, useCallback, useRef, useEffect } from 'react';
import type { WsServerMessage, WsClientMessage, Message } from '../types';

export const useChatStream = (sessionId: string | null) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const streamBufferRef = useRef<string>("");

  const connect = useCallback((apiKey: string) => {
    if (!sessionId) return;

    const host = import.meta.env.VITE_WS_BASE_URL || window.location.host;
    
    let wsUrl: string;
    if (host.startsWith('ws://') || host.startsWith('wss://')) {
        // If it's a full URL, use it as base
        wsUrl = `${host}/ws/chat/${sessionId}?api_key=${apiKey}`;
    } else {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        wsUrl = `${protocol}//${host}/ws/chat/${sessionId}?api_key=${apiKey}`;
    }
    
    // Clean up double slashes (except after protocol)
    wsUrl = wsUrl.replace(/([^:]\/)\/+/g, "$1");
    console.log('Connecting to WebSocket:', wsUrl);

    if (wsRef.current) wsRef.current.close();

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('WS Connected');
      setError(null);
    };

    ws.onmessage = (event) => {
      const data: WsServerMessage = JSON.parse(event.data);

      if (data.type === 'chunk') {
        setIsStreaming(true);
        streamBufferRef.current += data.content;
        
        // Update the last assistant message in state or add a temporary one
        setMessages(prev => {
          const last = prev[prev.length - 1];
          if (last && last.role === 'assistant') {
            return [
              ...prev.slice(0, -1),
              { ...last, content: streamBufferRef.current }
            ];
          } else {
            return [
              ...prev,
              { 
                id: Date.now(), 
                session_id: sessionId, 
                role: 'assistant', 
                content: data.content,
                model: 'streaming...',
                token_count: null,
                created_at: new Date().toISOString(),
                metadata: {}
              } as Message
            ];
          }
        });
      } else if (data.type === 'done') {
        setIsStreaming(false);
        streamBufferRef.current = "";
      } else if (data.type === 'error') {
        setError(data.content);
        setIsStreaming(false);
      }
    };

    ws.onerror = () => {
      setError('WebSocket connection error');
      setIsStreaming(false);
    };

    ws.onclose = () => {
      console.log('WS Disconnected');
      setIsStreaming(false);
    };
  }, [sessionId]);

  const sendMessage = useCallback((content: string) => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      setError('Not connected');
      return;
    }

    const clientMsg: WsClientMessage = {
      type: 'message',
      content,
      stream: true
    };

    // Optimistically add user message
    setMessages(prev => [
      ...prev,
      {
        id: Date.now(),
        session_id: sessionId!,
        role: 'user',
        content,
        model: null,
        token_count: null,
        created_at: new Date().toISOString(),
        metadata: {}
      } as Message
    ]);

    wsRef.current.send(JSON.stringify(clientMsg));
  }, [sessionId]);

  useEffect(() => {
    return () => {
      if (wsRef.current) wsRef.current.close();
    };
  }, []);

  return {
    messages,
    setMessages,
    isStreaming,
    error,
    connect,
    sendMessage
  };
};
