import React, { useEffect, useState } from 'react';
import { getMcpTools, type McpTool } from '../api/llm';
import { Wrench, Box, Code } from 'lucide-react';
import { motion } from 'framer-motion';

const McpTools: React.FC = () => {
  const [tools, setTools] = useState<McpTool[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getMcpTools()
      .then(setTools)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-monokai-pink"></div>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="flex items-center gap-4 mb-8">
        <Wrench className="w-10 h-10 text-monokai-pink" />
        <h1 className="text-3xl font-bold text-gruv-light-0">MCP Tool Registry</h1>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {tools.map((tool) => (
          <motion.div
            key={tool.name}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="glass p-6 rounded-xl hover:border-monokai-pink transition-colors group"
          >
            <div className="flex items-center gap-3 mb-4">
              <div className="p-2 rounded-lg bg-gruv-dark-3 group-hover:bg-monokai-pink/20 transition-colors">
                <Box className="w-6 h-6 text-monokai-aqua" />
              </div>
              <h3 className="text-xl font-bold text-gruv-light-1">{tool.name}</h3>
            </div>
            
            <p className="text-gruv-light-4 text-sm mb-6 line-clamp-2 italic">
              {tool.description}
            </p>

            <div className="space-y-3">
              <div className="flex items-center gap-2 text-xs font-mono text-gruv-gray">
                <Code className="w-4 h-4" />
                Input Schema
              </div>
              <pre className="p-3 bg-gruv-dark-0 rounded-lg text-[10px] text-monokai-green overflow-x-auto border border-gruv-dark-4">
                {JSON.stringify(tool.input_schema, null, 2)}
              </pre>
            </div>
          </motion.div>
        ))}
      </div>

      {tools.length === 0 && (
        <div className="text-center py-20 bg-gruv-dark-2 rounded-2xl border border-dashed border-gruv-dark-4">
          <p className="text-gruv-light-4">No MCP tools registered in stepbit-core.</p>
        </div>
      )}
    </div>
  );
};

export default McpTools;
