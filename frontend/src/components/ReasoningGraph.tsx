import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';

export interface ReasoningNode {
  id: string;
  node_type: string;
  payload: any;
  status?: 'pending' | 'running' | 'success' | 'error';
  result?: any;
}

interface ReasoningGraphProps {
  graph: {
    nodes: Record<string, ReasoningNode>;
    edges: Array<{ from: string; to: string }>;
  };
}

const ReasoningGraph: React.FC<ReasoningGraphProps> = ({ graph }) => {
  const nodes = Object.values(graph.nodes);

  return (
    <div 
      data-testid="reasoning-graph-container"
      className="p-6 bg-gruv-dark-1 rounded-xl border border-gruv-dark-4 min-h-[400px]"
    >
      <div className="flex flex-wrap gap-4">
        <AnimatePresence>
          {nodes.map((node) => (
            <motion.div
              key={node.id}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              className={`p-4 rounded-lg border-2 w-64 ${
                node.node_type === 'LlmGeneration' ? 'border-monokai-aqua bg-monokai-aqua/10' :
                node.node_type === 'McpToolCall' ? 'border-monokai-green bg-monokai-green/10' :
                'border-gruv-gray bg-gruv-dark-2'
              }`}
            >
              <div className="flex justify-between items-center mb-2">
                <span className="font-mono text-sm font-bold text-gruv-light-1">{node.id}</span>
                <span className="text-[10px] px-2 py-0.5 rounded bg-gruv-dark-3 text-gruv-light-4 uppercase">
                  {node.node_type}
                </span>
              </div>
              <div className="text-sm text-gruv-light-2 truncate">
                {JSON.stringify(node.payload)}
              </div>
              {node.status && (
                <div className="mt-3 flex items-center gap-2">
                   <div className={`w-2 h-2 rounded-full ${
                     node.status === 'running' ? 'bg-monokai-orange animate-pulse' :
                     node.status === 'success' ? 'bg-monokai-green' :
                     node.status === 'error' ? 'bg-monokai-pink' :
                     'bg-gruv-gray'
                   }`} />
                   <span className="text-[10px] font-bold text-gruv-light-4 uppercase">{node.status}</span>
                </div>
              )}
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      {/* Edges visualization could be added here with SVG overlay */}
      {graph.edges.length > 0 && (
         <div className="mt-8 text-xs text-gruv-gray font-mono">
            Edges: {graph.edges.map(e => `${e.from} → ${e.to}`).join(', ')}
         </div>
      )}
    </div>
  );
};

export default ReasoningGraph;
