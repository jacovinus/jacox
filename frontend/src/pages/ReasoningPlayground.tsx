import React, { useState } from 'react';
import ReasoningGraph from '../components/ReasoningGraph';
import { executeReasoning } from '../api/llm';
import { Play, Plus, Zap, Trash2 } from 'lucide-react';
import { motion } from 'framer-motion';

const ReasoningPlayground: React.FC = () => {
  const [graph, setGraph] = useState({
    nodes: {
      'node-1': { id: 'node-1', node_type: 'LlmGeneration', payload: { prompt: 'Why is the sky blue?' } }
    },
    edges: []
  });
  const [results, setResults] = useState<any>(null);
  const [executing, setExecuting] = useState(false);

  const handleExecute = async () => {
    setExecuting(true);
    try {
      const res = await executeReasoning(graph);
      setResults(res);
    } catch (e) {
      console.error(e);
      alert('Execution failed');
    } finally {
      setExecuting(false);
    }
  };

  const addNode = () => {
    const id = `node-${Object.keys(graph.nodes).length + 1}`;
    setGraph({
      ...graph,
      nodes: {
        ...graph.nodes,
        [id]: { id, node_type: 'LlmGeneration', payload: { prompt: 'New Task' } }
      }
    });
  };

  return (
    <div className="p-8 max-w-7xl mx-auto">
      <div className="flex justify-between items-center mb-8">
        <div className="flex items-center gap-4">
          <Zap className="w-10 h-10 text-monokai-orange" />
          <h1 className="text-3xl font-bold text-gruv-light-0">Reasoning Playground</h1>
        </div>
        
        <div className="flex gap-3">
          <button 
            onClick={addNode}
            className="btn-secondary flex items-center gap-2"
          >
            <Plus className="w-4 h-4" /> Add Node
          </button>
          <button 
            onClick={handleExecute}
            disabled={executing}
            className="btn-primary flex items-center gap-2"
          >
            <Play className={`w-4 h-4 ${executing ? 'animate-spin' : ''}`} />
            {executing ? 'Executing...' : 'Run Graph'}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <motion.div 
          className="lg:col-span-2"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
        >
          <ReasoningGraph graph={graph} />
        </motion.div>

        <div className="space-y-6">
          <div className="glass p-6 rounded-xl">
            <h3 className="text-lg font-bold mb-4 flex items-center gap-2">
               <Trash2 className="w-4 h-4 text-monokai-pink" />
               Graph Results
            </h3>
            {results ? (
              <pre className="text-[10px] font-mono p-4 bg-gruv-dark-0 rounded-lg border border-gruv-dark-4 overflow-auto max-h-[400px]">
                {JSON.stringify(results, null, 2)}
              </pre>
            ) : (
              <p className="text-sm text-gruv-gray italic">No results yet. Run the graph to see output.</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ReasoningPlayground;
