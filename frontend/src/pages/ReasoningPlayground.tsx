import React, { useState } from 'react';
import ReasoningGraph, { type ReasoningNode } from '../components/ReasoningGraph';
import { executeReasoningStream } from '../api/llm';
import { Play, Zap, Trash2, X, Settings2, Database, Wrench, Brain, List, CheckCircle } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

const ReasoningPlayground: React.FC = () => {
  const [nodes, setNodes] = useState<Record<string, ReasoningNode>>({
    'node-1': { 
      id: 'node-1', 
      node_type: 'LlmGeneration', 
      payload: { prompt: 'Who is the CEO of Google?' },
      position: { x: 50, y: 50 }
    },
    'node-2': { 
      id: 'node-2', 
      node_type: 'Summarization', 
      payload: { input: '{{node-1.output}}' },
      position: { x: 350, y: 150 }
    }
  });

  const [edges, setEdges] = useState<Array<{ from: string; to: string }>>([
    { from: 'node-1', to: 'node-2' }
  ]);

  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [executing, setExecuting] = useState(false);
  const [results, setResults] = useState<Record<string, any>>({});

  const handleNodeMove = (id: string, x: number, y: number) => {
    setNodes(prev => ({
      ...prev,
      [id]: { ...prev[id], position: { x, y } }
    }));
  };

  const addNode = (type: string) => {
    const id = `node-${Date.now().toString().slice(-4)}`;
    const newNode: ReasoningNode = {
      id,
      node_type: type,
      payload: type === 'LlmGeneration' ? { prompt: '' } : 
               type === 'McpToolCall' ? { tool: '', input: {} } :
               type === 'DataQuery' ? { tool: 'duckdb_query', input: { query: '' } } : {},
      position: { x: 100, y: 100 },
      status: 'pending'
    };
    setNodes(prev => ({ ...prev, [id]: newNode }));
    setSelectedNodeId(id);
  };

  const deleteNode = (id: string) => {
    const newNodes = { ...nodes };
    delete newNodes[id];
    setNodes(newNodes);
    setEdges(prev => prev.filter(e => e.from !== id && e.to !== id));
    if (selectedNodeId === id) setSelectedNodeId(null);
  };

  const addEdge = (fromId: string, toId: string) => {
    if (fromId === toId) return;
    if (edges.some(e => e.from === fromId && e.to === toId)) return;
    setEdges(prev => [...prev, { from: fromId, to: toId }]);
  };

  const removeEdge = (index: number) => {
    setEdges(prev => prev.filter((_, i) => i !== index));
  };

  const handleExecute = async () => {
    setExecuting(true);
    setResults({});
    
    // Reset statuses and marks as running initially or as progress allows
    setNodes(prev => {
        const reset = { ...prev };
        Object.keys(reset).forEach(k => {
          reset[k] = { ...reset[k], status: 'pending', result: undefined };
        });
        return reset;
    });

    try {
      const graphData = { nodes: { ...nodes }, edges: [ ...edges ] };
      console.log('Streaming Reasoning Graph:', graphData);
      
      await executeReasoningStream(graphData, (event: any) => {
        if (event.type === 'node_completed') {
          const { node_id, result } = event;
          setResults(prev => ({ ...prev, [node_id]: result }));
          setNodes(prev => ({
            ...prev,
            [node_id]: { ...prev[node_id], status: 'success', result }
          }));
        } else if (event.type === 'node_started') {
          const { node_id } = event;
          setNodes(prev => ({
            ...prev,
            [node_id]: { ...prev[node_id], status: 'running' }
          }));
        } else if (event.type === 'error') {
          console.error('Stream node error:', event.error);
          alert(`Error in node: ${event.error}`);
        }
      });
      
    } catch (e: any) {
      console.error('Execution error:', e);
      alert(`Execution failed: ${e.message || 'Unknown error'}. Check the console for details.`);
    } finally {
      setExecuting(false);
    }
  };

  const clearGraph = () => {
    setNodes({});
    setEdges([]);
    setSelectedNodeId(null);
    setResults({});
  };

  const selectedNode = selectedNodeId ? nodes[selectedNodeId] : null;

  return (
    <div className="min-h-screen bg-gruv-dark-0 flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-gruv-dark-3 flex justify-between items-center bg-gruv-dark-1">
        <div className="flex items-center gap-4">
          <Zap className="w-8 h-8 text-monokai-orange" />
          <h1 className="text-2xl font-bold text-gruv-light-0">Reasoning Playground</h1>
        </div>
        
        <div className="flex gap-3">
          <button 
            onClick={clearGraph}
            className="btn-secondary flex items-center gap-2 px-4 border-monokai-pink/30 text-monokai-pink/70 hover:bg-monokai-pink/10"
          >
            <Trash2 className="w-4 h-4" /> Clear
          </button>
          <div className="flex p-1 bg-gruv-dark-0 rounded-lg border border-gruv-dark-4 mr-4">
            {[
              { type: 'LlmGeneration', icon: Brain, color: 'text-monokai-aqua' },
              { type: 'McpToolCall', icon: Wrench, color: 'text-monokai-green' },
              { type: 'DataQuery', icon: Database, color: 'text-monokai-orange' },
              { type: 'Summarization', icon: List, color: 'text-gruv-light-3' },
              { type: 'Verification', icon: CheckCircle, color: 'text-monokai-pink' },
            ].map(tool => (
              <button
                key={tool.type}
                onClick={() => addNode(tool.type)}
                className={`p-2 hover:bg-gruv-dark-3 rounded transition-colors ${tool.color}`}
                title={`Add ${tool.type}`}
              >
                <tool.icon size={18} />
              </button>
            ))}
          </div>

          <button 
            onClick={handleExecute}
            disabled={executing}
            className="btn-primary flex items-center gap-2 px-6"
          >
            <Play className={`w-4 h-4 ${executing ? 'animate-spin' : ''}`} />
            {executing ? 'Executing...' : 'Run Graph'}
          </button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* Canvas Area */}
        <div className="flex-1 relative overflow-auto p-4 flex flex-col gap-4">
           <ReasoningGraph 
             nodes={nodes} 
             edges={edges} 
             onNodeMove={handleNodeMove}
             onNodeSelect={setSelectedNodeId}
             selectedNodeId={selectedNodeId || undefined}
           />
           
            {/* Results Preview (Vertical Log) */}
            <div className="bg-gruv-dark-1 rounded-xl border border-gruv-dark-3 flex flex-col h-64">
              <div className="p-4 border-b border-gruv-dark-3 flex justify-between items-center bg-gruv-dark-1/50">
                <h3 className="text-gruv-light-4 font-bold text-xs uppercase tracking-widest">Execution Log</h3>
                <span className="text-[10px] text-gruv-gray font-mono">{Object.keys(results).length} items</span>
              </div>
              <div className="flex-1 overflow-y-auto p-2 space-y-1 font-mono text-xs">
                {Object.keys(results).length === 0 && !executing && (
                  <div className="h-full flex items-center justify-center text-gruv-gray italic">
                    Run the graph to see live execution traces...
                  </div>
                )}
                {Object.entries(results).reverse().map(([id, result]) => (
                  <div key={id} className="p-3 bg-gruv-dark-0 rounded border border-gruv-dark-4 hover:border-monokai-aqua/50 transition-colors group">
                    <div className="flex justify-between items-center mb-1">
                      <div className="text-monokai-aqua font-bold">{id}</div>
                      <div className="text-[10px] text-monokai-green bg-monokai-green/10 px-1.5 py-0.5 rounded">SUCCESS</div>
                    </div>
                    <div className="text-gruv-light-3 break-words whitespace-pre-wrap">
                       {typeof result.output === 'string' ? result.output : JSON.stringify(result, null, 2)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
        </div>

        {/* Sidebar */}
        <AnimatePresence>
          {selectedNodeId && selectedNode && (
            <motion.div 
              initial={{ x: 300 }}
              animate={{ x: 0 }}
              exit={{ x: 300 }}
              className="w-[450px] bg-gruv-dark-1 border-l border-gruv-dark-3 p-6 space-y-6 overflow-y-auto"
            >
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  <Settings2 className="w-5 h-5 text-gruv-light-4" />
                  <h3 className="text-lg font-bold text-gruv-light-0">Node Inspector</h3>
                </div>
                <button onClick={() => setSelectedNodeId(null)} className="text-gruv-gray hover:text-gruv-light-1">
                  <X size={20} />
                </button>
              </div>

              <div className="space-y-6">
                {/* Status Section */}
                {selectedNode.status && (
                  <div className={`p-3 rounded-lg border flex items-center justify-between ${
                    selectedNode.status === 'success' ? 'bg-monokai-green/10 border-monokai-green/30 text-monokai-green' :
                    selectedNode.status === 'running' ? 'bg-monokai-orange/10 border-monokai-orange/30 text-monokai-orange' :
                    'bg-gruv-dark-0 border-gruv-dark-4 text-gruv-gray'
                  }`}>
                    <span className="text-xs font-bold uppercase tracking-wider">Status: {selectedNode.status}</span>
                    <div className={`w-2 h-2 rounded-full ${
                      selectedNode.status === 'success' ? 'bg-monokai-green' :
                      selectedNode.status === 'running' ? 'bg-monokai-orange animate-pulse' :
                      'bg-gruv-gray'
                    }`} />
                  </div>
                )}

                {/* Configuration */}
                <div>
                  <label className="text-[10px] font-bold text-gruv-light-4 block mb-2 uppercase tracking-widest">Configuration (JSON)</label>
                  <textarea 
                    className="w-full h-40 bg-gruv-dark-0 border border-gruv-dark-4 font-mono text-xs text-monokai-aqua p-3 rounded outline-none focus:border-monokai-aqua transition-colors"
                    value={JSON.stringify(selectedNode.payload, null, 2)}
                    onChange={(e) => {
                      try {
                        const newPayload = JSON.parse(e.target.value);
                        setNodes(prev => ({
                          ...prev,
                          [selectedNodeId]: { ...prev[selectedNodeId], payload: newPayload }
                        }));
                      } catch (err) { }
                    }}
                  />
                </div>

                {/* Result Viewer */}
                {results[selectedNodeId] && (
                  <div className="space-y-2">
                    <label className="text-[10px] font-bold text-gruv-light-4 block uppercase tracking-widest">Execution Result</label>
                    <div className="w-full bg-gruv-dark-0 border border-monokai-green/30 p-4 rounded-lg font-mono text-xs overflow-x-auto max-h-96">
                      <pre className="text-monokai-green whitespace-pre-wrap">
                        {typeof results[selectedNodeId].output === 'string' 
                          ? results[selectedNodeId].output 
                          : JSON.stringify(results[selectedNodeId], null, 2)}
                      </pre>
                    </div>
                  </div>
                )}

                <div className="pt-4 border-t border-gruv-dark-4">
                  <label className="text-[10px] font-bold text-gruv-light-4 block mb-3 uppercase tracking-widest">Outbound Connections</label>
                  <div className="space-y-3">
                    <div className="flex gap-2">
                      <select 
                        className="flex-1 bg-gruv-dark-0 border border-gruv-dark-4 text-sm text-gruv-light-2 p-2 rounded focus:border-monokai-aqua outline-none"
                        onChange={(e) => addEdge(selectedNodeId, e.target.value)}
                        value=""
                      >
                        <option value="">Add connection to...</option>
                        {Object.keys(nodes)
                          .filter(id => id !== selectedNodeId)
                          .map(id => <option key={id} value={id}>{id}</option>)}
                      </select>
                    </div>
                    
                    <div className="flex flex-wrap gap-2">
                       {edges.filter(e => e.from === selectedNodeId || e.to === selectedNodeId).map((e, idx) => (
                         <div key={idx} className="flex items-center gap-2 text-[10px] bg-gruv-dark-0 px-2 py-1.5 rounded border border-gruv-dark-4 text-gruv-light-3">
                            <span className="font-mono">{e.from} → {e.to}</span>
                            <button 
                              onClick={() => removeEdge(edges.indexOf(e))}
                              className="text-monokai-pink hover:text-white transition-colors"
                            >
                                <X size={12}/>
                            </button>
                         </div>
                       ))}
                    </div>
                  </div>
                </div>

                <div className="pt-6">
                  <button 
                    onClick={() => deleteNode(selectedNodeId)}
                    className="w-full p-3 bg-monokai-pink/5 border border-monokai-pink/30 text-monokai-pink rounded-lg text-sm font-bold flex items-center justify-center gap-2 hover:bg-monokai-pink/20 transition-all"
                  >
                    <Trash2 size={16}/> Delete Node
                  </button>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default ReasoningPlayground;
