import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  Plus, 
  Play, 
  Settings, 
  Trash2, 
  Activity, 
  Info, 
  XCircle,
  RefreshCw,
  Search,
  Code,
  Layers
} from 'lucide-react';
import { pipelinesApi, type Pipeline } from '../api/pipelines';
import { useLlmos } from '../hooks/useLlmos';

const Pipelines: React.FC = () => {
  const [pipelines, setPipelines] = useState<Pipeline[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const { online, loading: statusLoading, refresh: refreshStatus } = useLlmos();

  // Modal States
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isJsonModalOpen, setIsJsonModalOpen] = useState(false);
  const [isExecutionModalOpen, setIsExecutionModalOpen] = useState(false);
  const [selectedPipeline, setSelectedPipeline] = useState<Pipeline | null>(null);
  
  // Create Form State
  const [newName, setNewName] = useState('');
  const [newDefinition, setNewDefinition] = useState('{\n  "name": "New Pipeline",\n  "rlm_enabled": false,\n  "stages": []\n}');

  // Execution State
  const [executionQuestion, setExecutionQuestion] = useState('');
  const [executionResult, setExecutionResult] = useState<any>(null);
  const [executing, setExecuting] = useState(false);

  useEffect(() => {
    loadPipelines();
  }, []);

  const loadPipelines = async () => {
    setLoading(true);
    try {
      const data = await pipelinesApi.list();
      setPipelines(data);
    } catch (error) {
      console.error('Failed to load pipelines:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async () => {
    try {
      const definition = JSON.parse(newDefinition);
      await pipelinesApi.create(newName, definition);
      setIsCreateModalOpen(false);
      setNewName('');
      loadPipelines();
    } catch (error) {
      alert('Invalid JSON definition or server error');
    }
  };

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this pipeline?')) {
      try {
        await pipelinesApi.delete(id);
        loadPipelines();
      } catch (error) {
        console.error('Failed to delete:', error);
      }
    }
  };

  const handleExecute = async () => {
    if (!selectedPipeline || !executionQuestion) return;
    setExecuting(true);
    setExecutionResult(null);
    try {
      const res = await pipelinesApi.execute(selectedPipeline.id, executionQuestion);
      setExecutionResult(res);
    } catch (error: any) {
      console.error('Execution failed:', error);
      const errorMsg = error.response?.data || error.message || 'Failed to execute pipeline';
      setExecutionResult({ error: typeof errorMsg === 'string' ? errorMsg : JSON.stringify(errorMsg) });
    } finally {
      setExecuting(false);
    }
  };

  const filteredPipelines = pipelines.filter(p => 
    p.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-7xl mx-auto min-h-screen">
      {/* Header & Status */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-12">
        <div className="flex items-center gap-4">
          <div className="p-3 rounded-2xl bg-monokai-orange/20 border border-monokai-orange/30">
            <Layers className="w-8 h-8 text-monokai-orange" />
          </div>
          <div>
            <h1 className="text-4xl font-black text-gruv-light-0 tracking-tight">Cognitive Pipelines</h1>
            <p className="text-gruv-light-4 font-medium">Orchestrate multi-stage LLM reasoning workflows</p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className={`flex items-center gap-2 px-4 py-2 rounded-full border glass ${online ? 'border-monokai-green/30 bg-monokai-green/5' : 'border-monokai-pink/30 bg-monokai-pink/5'}`}>
            <Activity className={`w-4 h-4 ${online ? 'text-monokai-green animate-pulse' : 'text-monokai-pink'}`} />
            <span className={`text-sm font-bold ${online ? 'text-monokai-green' : 'text-monokai-pink'}`}>
              LLMOS: {online ? 'Connected' : 'Disconnected'}
            </span>
            <button 
              onClick={refreshStatus}
              className="ml-2 p-1 hover:bg-white/10 rounded-full transition-colors"
              disabled={statusLoading}
            >
              <RefreshCw className={`w-3 h-3 text-gruv-light-4 ${statusLoading ? 'animate-spin' : ''}`} />
            </button>
          </div>
          
          <button 
            onClick={() => setIsCreateModalOpen(true)}
            className="flex items-center gap-2 bg-gradient-to-r from-monokai-orange to-monokai-pink text-white px-6 py-3 rounded-xl font-bold shadow-lg shadow-monokai-pink/20 hover:scale-105 transition-transform active:scale-95"
          >
            <Plus className="w-5 h-5" />
            Create Pipeline
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-col gap-8">
        {/* Search & Stats */}
        <div className="flex items-center gap-4 glass p-2 rounded-2xl border-white/5">
          <div className="relative flex-1">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gruv-light-4" />
            <input 
              type="text"
              placeholder="Search pipelines..."
              className="w-full bg-transparent border-none focus:ring-0 pl-12 py-3 text-gruv-light-1 placeholder-gruv-light-4 font-medium"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <div className="h-8 w-[1px] bg-white/10 mx-2" />
          <div className="px-6 py-2 text-sm font-bold text-gruv-light-3">
            {filteredPipelines.length} Total
          </div>
        </div>

        {/* Pipeline Grid */}
        {loading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {[1, 2, 3].map(i => (
              <div key={i} className="glass h-[200px] rounded-3xl animate-pulse bg-white/5" />
            ))}
          </div>
        ) : filteredPipelines.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredPipelines.map((pipeline) => (
              <PipelineCard 
                key={pipeline.id} 
                pipeline={pipeline} 
                disabled={!online}
                onExecute={() => {
                  setSelectedPipeline(pipeline);
                  setExecutionQuestion('');
                  setExecutionResult(null);
                  setIsExecutionModalOpen(true);
                }}
                onView={() => {
                  setSelectedPipeline(pipeline);
                  setIsJsonModalOpen(true);
                }}
                onDelete={() => handleDelete(pipeline.id)}
              />
            ))}
          </div>
        ) : (
          <div className="text-center py-32 glass rounded-[3rem] border-dashed border-white/10">
            <div className="bg-white/5 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-6">
              <Info className="w-10 h-10 text-gruv-light-4" />
            </div>
            <h3 className="text-2xl font-bold text-gruv-light-1 mb-2">No pipelines found</h3>
            <p className="text-gruv-light-4 max-w-md mx-auto">
              Create your first cognitive pipeline to start solving complex tasks with structured reasoning.
            </p>
          </div>
        )}
      </div>

      {/* Modals */}
      <AnimatePresence>
        {/* Create Modal */}
        {isCreateModalOpen && (
          <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <motion.div 
              initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              onClick={() => setIsCreateModalOpen(false)}
              className="absolute inset-0 bg-black/80 backdrop-blur-md"
            />
            <motion.div 
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative w-full max-w-2xl glass border-white/10 p-8 rounded-[2rem] shadow-2xl overflow-hidden"
            >
              <h2 className="text-3xl font-black text-gruv-light-1 mb-6 flex items-center gap-3">
                <Plus className="w-8 h-8 text-monokai-orange" />
                New Pipeline
              </h2>
              <div className="space-y-6">
                <div>
                  <label className="block text-xs font-black text-gruv-light-4 uppercase tracking-widest mb-2">Pipeline Name</label>
                  <input 
                    type="text" 
                    value={newName} 
                    onChange={e => setNewName(e.target.value)}
                    className="w-full bg-black/40 border border-white/5 rounded-xl px-4 py-3 text-gruv-light-1 focus:ring-2 focus:ring-monokai-orange outline-none"
                    placeholder="Enter process name..."
                  />
                </div>
                <div>
                  <label className="block text-xs font-black text-gruv-light-4 uppercase tracking-widest mb-2">JSON Definition</label>
                  <textarea 
                    value={newDefinition} 
                    onChange={e => setNewDefinition(e.target.value)}
                    rows={10}
                    className="w-full bg-black/40 border border-white/5 rounded-xl px-4 py-3 text-mono text-sm text-monokai-aqua font-mono focus:ring-2 focus:ring-monokai-orange outline-none"
                  />
                </div>
                <div className="flex gap-4">
                  <button 
                    onClick={() => setIsCreateModalOpen(false)}
                    className="flex-1 px-6 py-4 rounded-xl font-bold text-gruv-light-3 hover:bg-white/5 transition-colors"
                  >
                    Cancel
                  </button>
                  <button 
                    onClick={handleCreate}
                    className="flex-[2] bg-monokai-orange text-white px-6 py-4 rounded-xl font-black hover:brightness-110 transition-all shadow-lg shadow-monokai-orange/20"
                  >
                    Register Pipeline
                  </button>
                </div>
              </div>
            </motion.div>
          </div>
        )}

        {/* View JSON Modal */}
        {isJsonModalOpen && selectedPipeline && (
          <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <motion.div 
              initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              onClick={() => setIsJsonModalOpen(false)}
              className="absolute inset-0 bg-black/80 backdrop-blur-md"
            />
            <motion.div 
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative w-full max-w-2xl glass border-white/10 p-8 rounded-[2rem] shadow-2xl overflow-hidden"
            >
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-black text-gruv-light-1 flex items-center gap-3">
                  <Code className="w-6 h-6 text-monokai-aqua" />
                  {selectedPipeline.name}
                </h2>
                <button onClick={() => setIsJsonModalOpen(false)} className="p-2 hover:bg-white/10 rounded-full transition-colors text-gruv-light-4">
                  <Play className="w-5 h-5 rotate-45" />
                </button>
              </div>
              <div className="bg-black/60 rounded-2xl p-6 overflow-auto max-h-[60vh] border border-white/5">
                <pre className="text-monokai-aqua font-mono text-sm leading-relaxed">
                  {JSON.stringify(selectedPipeline.definition, null, 2)}
                </pre>
              </div>
              <div className="mt-8 flex justify-end">
                <button 
                  onClick={() => setIsJsonModalOpen(false)}
                  className="px-8 py-3 bg-white/5 hover:bg-white/10 rounded-xl font-bold text-gruv-light-1 transition-colors"
                >
                  Close
                </button>
              </div>
            </motion.div>
          </div>
        )}

        {/* Execution Modal */}
        {isExecutionModalOpen && selectedPipeline && (
          <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
            <motion.div 
              initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
              onClick={() => !executing && setIsExecutionModalOpen(false)}
              className="absolute inset-0 bg-black/80 backdrop-blur-md"
            />
            <motion.div 
              initial={{ opacity: 0, scale: 0.9, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.9, y: 20 }}
              className="relative w-full max-w-3xl glass border-white/10 p-8 rounded-[2rem] shadow-2xl overflow-hidden"
            >
              <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-monokai-green/20">
                    <Play className="w-6 h-6 text-monokai-green fill-monokai-green" />
                  </div>
                  <div>
                    <h2 className="text-2xl font-black text-gruv-light-1 tracking-tight">Run Pipeline</h2>
                    <p className="text-xs font-bold text-gruv-light-4 uppercase tracking-widest">{selectedPipeline.name}</p>
                  </div>
                </div>
                {!executing && (
                  <button onClick={() => setIsExecutionModalOpen(false)} className="p-2 hover:bg-white/10 rounded-full transition-colors text-gruv-light-4">
                    <XCircle className="w-6 h-6" />
                  </button>
                )}
              </div>

              <div className="space-y-6">
                {!executionResult && (
                  <div className="space-y-4">
                    <label className="block text-xs font-black text-gruv-light-4 uppercase tracking-widest px-1">Prompt / Question</label>
                    <textarea 
                      value={executionQuestion} 
                      onChange={e => setExecutionQuestion(e.target.value)}
                      placeholder="What should this pipeline solve?"
                      rows={3}
                      disabled={executing}
                      className="w-full bg-white/5 border border-white/5 rounded-2xl px-6 py-4 text-gruv-light-1 focus:ring-2 focus:ring-monokai-green outline-none min-h-[120px] resize-none"
                    />
                    <button 
                      onClick={handleExecute}
                      disabled={executing || !executionQuestion}
                      className={`w-full py-4 rounded-2xl font-black text-lg transition-all flex items-center justify-center gap-3 ${executing ? 'bg-white/5 text-gruv-light-4 cursor-wait' : 'bg-monokai-green text-black hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-monokai-green/20'}`}
                    >
                      {executing ? (
                        <>
                          <RefreshCw className="w-5 h-5 animate-spin" />
                          Processing reasoning steps...
                        </>
                      ) : (
                        <>
                          <Play className="w-5 h-5 fill-current" />
                          Initiate Sequence
                        </>
                      )}
                    </button>
                  </div>
                )}

                {executionResult && (
                  <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    {/* Error Display */}
                    {executionResult.error && (
                      <div className="bg-monokai-pink/10 border border-monokai-pink/20 rounded-2xl p-6">
                        <div className="flex items-center gap-3 text-monokai-pink mb-2">
                          <XCircle className="w-5 h-5" />
                          <span className="text-xs font-black uppercase tracking-widest">Execution Failure</span>
                        </div>
                        <p className="text-gruv-light-1 text-sm font-medium">
                          {executionResult.error}
                        </p>
                      </div>
                    )}

                    {/* Trace Viewer */}
                    {!executionResult.error && (
                      <div className="space-y-4">
                        <div className="flex items-center gap-2 text-monokai-aqua">
                          <Activity className="w-4 h-4" />
                          <span className="text-xs font-black uppercase tracking-widest">Reasoning Trace</span>
                        </div>
                        <div className="bg-black/60 rounded-2xl p-6 space-y-3 max-h-[250px] overflow-auto border border-white/5">
                          {executionResult.trace?.map((log: string, i: number) => (
                            <div key={i} className="flex gap-4 items-start group">
                              <span className="text-[10px] font-mono text-gruv-light-4 mt-1">0{i+1}</span>
                              <p className="text-sm font-medium text-gruv-light-3 leading-relaxed group-hover:text-gruv-light-1 transition-colors">{log}</p>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Final Answer */}
                    {!executionResult.error && (
                      <div className="space-y-4">
                        <div className="flex items-center gap-2 text-monokai-orange">
                          <Info className="w-4 h-4" />
                          <span className="text-xs font-black uppercase tracking-widest">Final Synthesis</span>
                        </div>
                        <div className="bg-monokai-orange/5 border border-monokai-orange/20 rounded-2xl p-6">
                          <p className="text-gruv-light-1 font-medium leading-[1.6]">
                            {executionResult.final_answer || "No final answer synthesised."}
                          </p>
                        </div>
                      </div>
                    )}

                    <div className="flex gap-4">
                      <button 
                        onClick={() => { setExecutionResult(null); setExecutionQuestion(''); }}
                        className="flex-1 px-6 py-4 rounded-xl font-bold text-gruv-light-3 hover:bg-white/5 transition-colors"
                      >
                        Reset
                      </button>
                      <button 
                        onClick={() => setIsExecutionModalOpen(false)}
                        className="flex-1 bg-white text-black px-6 py-4 rounded-xl font-black hover:brightness-110 transition-all"
                      >
                        Done
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </motion.div>
          </div>
        )}
      </AnimatePresence>

      {/* Connection Warning Overlay */}
      <AnimatePresence>
        {!online && !loading && (
          <motion.div 
            initial={{ opacity: 0, y: 50 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 50 }}
            className="fixed bottom-8 left-1/2 -translate-x-1/2 glass px-8 py-4 rounded-2xl border-monokai-pink/50 bg-monokai-pink/10 flex items-center gap-4 z-50 shadow-2xl backdrop-blur-xl"
          >
            <XCircle className="w-6 h-6 text-monokai-pink" />
            <div>
              <p className="font-bold text-monokai-pink">LLMOS is currently offline</p>
              <p className="text-xs text-gruv-light-3 font-medium">Pipeline execution and tool access are limited.</p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

const PipelineCard: React.FC<{ 
  pipeline: Pipeline; 
  disabled: boolean;
  onExecute: () => void;
  onView: () => void;
  onDelete: () => void;
}> = ({ pipeline, disabled, onExecute, onView, onDelete }) => {
  return (
    <motion.div 
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      whileHover={{ y: -5 }}
      className={`glass group h-full rounded-3xl border-transparent hover:border-white/10 transition-all p-6 relative overflow-hidden flex flex-col ${disabled ? 'opacity-60 grayscale' : ''}`}
    >
      {/* Dynamic Background Gradient */}
      <div className="absolute top-0 right-0 w-32 h-32 bg-monokai-orange/10 blur-[60px] rounded-full -translate-y-1/2 translate-x-1/2 group-hover:bg-monokai-orange/20 transition-colors" />
      
      <div className="flex items-start justify-between mb-6 relative z-10">
        <div className="p-3 rounded-2xl bg-white/5 border border-white/5 group-hover:scale-110 transition-transform cursor-pointer" onClick={onView}>
          <Code className="w-6 h-6 text-monokai-aqua" />
        </div>
        <div className="flex gap-1">
          <button onClick={onView} className="p-2 hover:bg-white/10 rounded-xl transition-colors text-gruv-light-4 hover:text-gruv-light-1">
            <Settings className="w-4 h-4" />
          </button>
          <button onClick={onDelete} className="p-2 hover:bg-monokai-pink/20 rounded-xl transition-colors text-gruv-light-4 hover:text-monokai-pink">
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="flex-1 relative z-10">
        <h3 className="text-xl font-black text-gruv-light-1 mb-2 group-hover:text-monokai-aqua transition-colors cursor-pointer" onClick={onView}>
          {pipeline.name}
        </h3>
        <p className="text-xs font-mono text-gruv-light-4 mb-4">
          ID: {pipeline.id}
        </p>

        <div className="flex flex-wrap gap-2 mb-6 cursor-pointer" onClick={onView}>
          {pipeline.definition.stages.slice(0, 3).map((stage: any, i: number) => (
            <span key={i} className="px-2 py-1 rounded-lg bg-black/40 border border-white/5 text-[10px] font-bold text-monokai-green uppercase tracking-wider">
              {stage.stage_type.replace('Stage', '').replace('_stage', '')}
            </span>
          ))}
          {pipeline.definition.stages.length > 3 && (
            <span className="px-2 py-1 rounded-lg bg-black/40 border border-white/5 text-[10px] font-bold text-gruv-light-4">
              +{pipeline.definition.stages.length - 3} MORE
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between pt-4 border-t border-white/5 relative z-10">
        <div className="flex flex-col">
          <span className="text-[10px] font-black text-gruv-light-4 uppercase tracking-tighter">Stages</span>
          <span className="text-lg font-black text-gruv-light-1">{pipeline.definition.stages.length}</span>
        </div>
        
        <button 
          onClick={onExecute}
          disabled={disabled}
          className={`flex items-center gap-2 ${disabled ? 'bg-gruv-dark-2 text-gruv-light-4 cursor-not-allowed' : 'bg-white text-black hover:bg-monokai-aqua hover:text-black'} px-5 py-2.5 rounded-xl font-black text-xs uppercase transition-all shadow-xl active:scale-95`}
        >
          <Play className="w-4 h-4" />
          Execute
        </button>
      </div>
    </motion.div>
  );
};

export default Pipelines;
