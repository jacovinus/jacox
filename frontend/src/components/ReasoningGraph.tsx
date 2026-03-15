import React, { useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { type LucideIcon, Brain, Wrench, Database, CheckCircle, List, GitBranch } from 'lucide-react';

export interface ReasoningNode {
  id: string;
  node_type: string;
  payload: any;
  status?: 'pending' | 'running' | 'success' | 'error';
  result?: any;
  position?: { x: number; y: number };
}

interface ReasoningGraphProps {
  nodes: Record<string, ReasoningNode>;
  edges: Array<{ from: string; to: string }>;
  onNodeMove: (id: string, x: number, y: number) => void;
  onNodeSelect: (id: string) => void;
  selectedNodeId?: string;
}

const NODE_WIDTH = 240;
const NODE_HEIGHT = 100;

const NodeTypeIcon: Record<string, LucideIcon> = {
  LlmGeneration: Brain,
  McpToolCall: Wrench,
  DataQuery: Database,
  Verification: CheckCircle,
  Summarization: List,
  ConditionalBranch: GitBranch,
};

const ReasoningGraph: React.FC<ReasoningGraphProps> = ({ 
  nodes, 
  edges, 
  onNodeMove, 
  onNodeSelect,
  selectedNodeId 
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const getEdgePath = (fromId: string, toId: string) => {
    const from = nodes[fromId];
    const to = nodes[toId];
    if (!from || !to) return '';

    const startX = (from.position?.x || 0) + NODE_WIDTH;
    const startY = (from.position?.y || 0) + NODE_HEIGHT / 2;
    const endX = to.position?.x || 0;
    const endY = (to.position?.y || 0) + NODE_HEIGHT / 2;

    const controlX = startX + (endX - startX) / 2;
    
    return `M ${startX} ${startY} C ${controlX} ${startY}, ${controlX} ${endY}, ${endX} ${endY}`;
  };

  return (
    <div 
      ref={containerRef}
      data-testid="reasoning-graph-container"
      className="relative w-full h-[600px] bg-gruv-dark-0 rounded-xl border border-gruv-dark-3 overflow-hidden"
      style={{
        backgroundImage: 'radial-gradient(circle, #3c3836 1px, transparent 1px)',
        backgroundSize: '30px 30px'
      }}
    >
      <svg className="absolute inset-0 w-full h-full pointer-events-none">
        <defs>
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="7"
            refX="9"
            refY="3.5"
            orient="auto"
          >
            <polygon points="0 0, 10 3.5, 0 7" fill="#665c54" />
          </marker>
        </defs>
        {edges.map((edge, i) => (
          <path
            key={`${edge.from}-${edge.to}-${i}`}
            d={getEdgePath(edge.from, edge.to)}
            fill="none"
            stroke="#665c54"
            strokeWidth="2"
            markerEnd="url(#arrowhead)"
          />
        ))}
      </svg>

      <AnimatePresence>
        {Object.values(nodes).map((node) => {
          const Icon = NodeTypeIcon[node.node_type] || Brain;
          const isSelected = selectedNodeId === node.id;

          return (
            <motion.div
              key={node.id}
              drag
              dragMomentum={false}
              dragElastic={0}
              onDrag={(_, info) => {
                const currentX = node.position?.x || 0;
                const currentY = node.position?.y || 0;
                onNodeMove(node.id, currentX + info.delta.x, currentY + info.delta.y);
              }}
              initial={false}
              animate={{
                x: node.position?.x || 0,
                y: node.position?.y || 0,
              }}
              whileHover={{ scale: 1.02, cursor: 'grab' }}
              whileDrag={{ scale: 1.05, cursor: 'grabbing', zIndex: 50 }}
              onClick={() => onNodeSelect(node.id)}
              className={`absolute p-4 rounded-lg border-2 cursor-grab active:cursor-grabbing w-[240px] z-10 transition-shadow ${
                isSelected ? 'ring-2 ring-monokai-yellow shadow-lg shadow-monokai-yellow/20' : ''
              } ${
                node.node_type === 'LlmGeneration' ? 'border-monokai-aqua bg-gruv-dark-1' :
                node.node_type === 'McpToolCall' ? 'border-monokai-green bg-gruv-dark-1' :
                node.node_type === 'DataQuery' ? 'border-monokai-orange bg-gruv-dark-1' :
                'border-gruv-gray bg-gruv-dark-1'
              }`}
            >
              <div className="flex justify-between items-center mb-2">
                <div className="flex items-center gap-2">
                  <Icon size={16} className={
                    node.node_type === 'LlmGeneration' ? 'text-monokai-aqua' :
                    node.node_type === 'McpToolCall' ? 'text-monokai-green' :
                    'text-gruv-light-4'
                  } />
                  <span className="font-mono text-sm font-bold text-gruv-light-1 truncate max-w-[120px]">
                    {node.id}
                  </span>
                </div>
                <div className={`w-2 h-2 rounded-full ${
                  node.status === 'running' ? 'bg-monokai-orange animate-pulse' :
                  node.status === 'success' ? 'bg-monokai-green' :
                  node.status === 'error' ? 'bg-monokai-pink' :
                  'bg-gruv-dark-4'
                }`} />
              </div>
              
              <div className="text-[10px] text-gruv-light-4 uppercase font-bold mb-2">
                {node.node_type.replace(/([A-Z])/g, ' $1').trim()}
              </div>

              <div className="text-xs text-gruv-light-3 line-clamp-2 italic">
                {node.node_type === 'LlmGeneration' ? node.payload.prompt : 
                 node.node_type === 'McpToolCall' ? `Call ${node.payload.tool}` :
                 'Configure node...'}
              </div>
            </motion.div>
          );
        })}
      </AnimatePresence>
    </div>
  );
};

export default ReasoningGraph;
