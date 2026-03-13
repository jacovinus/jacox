import React, { useState, useEffect, useMemo } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { sql } from '@codemirror/lang-sql';
import { oneDark } from '@codemirror/theme-one-dark';
import { Play, Table as TableIcon, Database, Info, Loader2, AlertCircle, ChevronLeft, ChevronRight, Search } from 'lucide-react';
import api from '../api/client';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface QueryResponse {
  columns: string[];
  rows: Record<string, any>[];
}

interface SchemaInfo {
  table_name: string;
  columns: { name: string; type: string }[];
}

const DatabaseExplorer: React.FC = () => {
  const [query, setQuery] = useState('SELECT * FROM sessions LIMIT 10;');
  const [results, setResults] = useState<QueryResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [schema, setSchema] = useState<SchemaInfo[]>([]);
  const [schemaLoading, setSchemaLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'results' | 'schema'>('results');
  const [searchTerm, setSearchTerm] = useState('');

  // Pagination
  const [pageSize] = useState(10);
  const [currentPage, setCurrentPage] = useState(1);

  useEffect(() => {
    fetchSchema();
  }, []);

  const fetchSchema = async () => {
    setSchemaLoading(true);
    try {
      // DuckDB specific schema query
      const res = await api.post<QueryResponse>('/query', {
        sql: "SELECT table_name, column_name, data_type FROM information_schema.columns WHERE table_schema = 'main' ORDER BY table_name, ordinal_position;"
      });

      const tables: Record<string, SchemaInfo> = {};
      res.data.rows.forEach(row => {
        if (!tables[row.table_name]) {
          tables[row.table_name] = { table_name: row.table_name, columns: [] };
        }
        tables[row.table_name].columns.push({ name: row.column_name, type: row.data_type });
      });

      setSchema(Object.values(tables));
    } catch (err: any) {
      console.error('Failed to fetch schema:', err);
    } finally {
      setSchemaLoading(false);
    }
  };

  const handleRunQuery = async () => {
    setLoading(true);
    setError(null);
    setCurrentPage(1);
    try {
      const res = await api.post<QueryResponse>('/query', { sql: query });
      setResults(res.data);
      setActiveTab('results');
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Failed to execute query');
      setResults(null);
    } finally {
      setLoading(false);
    }
  };

  const filteredSchema = useMemo(() => {
    if (!searchTerm) return schema;
    return schema.filter(t => 
      t.table_name.toLowerCase().includes(searchTerm.toLowerCase()) || 
      t.columns.some(c => c.name.toLowerCase().includes(searchTerm.toLowerCase()))
    );
  }, [schema, searchTerm]);

  const paginatedRows = useMemo(() => {
    if (!results) return [];
    const start = (currentPage - 1) * pageSize;
    return results.rows.slice(start, start + pageSize);
  }, [results, currentPage, pageSize]);

  const totalPages = Math.ceil((results?.rows.length || 0) / pageSize);

  return (
    <div className="flex flex-col h-full bg-[#0a0a0a] text-zinc-100 p-6 gap-6 overflow-hidden">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-blue-500/10 rounded-lg">
            <Database className="w-6 h-6 text-blue-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold tracking-tight">DuckDB Explorer</h1>
            <p className="text-sm text-zinc-500">Query and explore your internal database</p>
          </div>
        </div>
        <button
          onClick={handleRunQuery}
          disabled={loading || !query.trim()}
          className={cn(
            "flex items-center gap-2 px-6 py-2.5 bg-blue-600 hover:bg-blue-500 disabled:bg-zinc-800 disabled:text-zinc-600 rounded-xl font-semibold transition-all shadow-lg shadow-blue-900/20 active:scale-95",
            loading && "cursor-not-allowed opacity-70"
          )}
        >
          {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
          Run Query
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 flex-1 min-h-0">
        {/* Editor & Results Panel */}
        <div className="lg:col-span-3 flex flex-col gap-6 min-h-0">
          <div className="bg-[#121212] border border-zinc-800 rounded-2xl overflow-hidden shadow-2xl flex flex-col h-1/2 min-h-[300px]">
            <div className="px-4 py-2 bg-zinc-900/50 border-b border-zinc-800 flex items-center justify-between">
              <span className="text-xs font-medium text-zinc-500 uppercase tracking-widest">SQL Editor</span>
              <div className="flex gap-1.5">
                <div className="w-2.5 h-2.5 rounded-full bg-zinc-800" />
                <div className="w-2.5 h-2.5 rounded-full bg-zinc-800" />
                <div className="w-2.5 h-2.5 rounded-full bg-zinc-800" />
              </div>
            </div>
            <div className="flex-1 overflow-auto custom-scrollbar">
              <CodeMirror
                value={query}
                height="100%"
                theme={oneDark}
                extensions={[sql()]}
                onChange={(value) => setQuery(value)}
                className="text-sm"
              />
            </div>
          </div>

          <div className="bg-[#121212] border border-zinc-800 rounded-2xl overflow-hidden shadow-2xl flex flex-col flex-1 min-h-0">
            <div className="px-4 border-b border-zinc-800 bg-zinc-900/50 flex items-center justify-between">
              <div className="flex">
                <button
                  onClick={() => setActiveTab('results')}
                  className={cn(
                    "px-6 py-3 text-sm font-medium transition-all relative",
                    activeTab === 'results' ? "text-blue-400" : "text-zinc-500 hover:text-zinc-300"
                  )}
                >
                  Results
                  {activeTab === 'results' && (
                    <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500" />
                  )}
                </button>
                <button
                  onClick={() => setActiveTab('schema')}
                  className={cn(
                    "px-6 py-3 text-sm font-medium transition-all relative lg:hidden",
                    activeTab === 'schema' ? "text-blue-400" : "text-zinc-500 hover:text-zinc-300"
                  )}
                >
                  Schema
                  {activeTab === 'schema' && (
                    <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500" />
                  )}
                </button>
              </div>
              {results && activeTab === 'results' && (
                <span className="text-xs text-zinc-500 px-4">
                  {results.rows.length} rows returned
                </span>
              )}
            </div>

            <div className="flex-1 overflow-hidden relative">
              {loading && (
                <div className="absolute inset-0 bg-black/40 backdrop-blur-[2px] z-10 flex items-center justify-center">
                  <div className="flex flex-col items-center gap-3">
                    <Loader2 className="w-10 h-10 text-blue-500 animate-spin" />
                    <span className="text-sm font-medium text-blue-400 animate-pulse">Executing transaction...</span>
                  </div>
                </div>
              )}

              {error && (
                <div className="p-8 flex flex-col items-center text-center max-w-md mx-auto h-full justify-center">
                  <div className="p-4 bg-red-500/10 rounded-full mb-4">
                    <AlertCircle className="w-10 h-10 text-red-500" />
                  </div>
                  <h3 className="text-lg font-bold text-red-400 mb-2">Query Execution Error</h3>
                  <p className="text-sm text-zinc-500 bg-zinc-900 p-4 rounded-xl border border-red-500/20 font-mono break-all line-clamp-4">
                    {error}
                  </p>
                </div>
              )}

              {results && activeTab === 'results' && (
                <div className="flex flex-col h-full">
                  <div className="flex-1 overflow-auto custom-scrollbar">
                    <table className="w-full text-left border-collapse min-w-full">
                      <thead className="sticky top-0 z-20 bg-[#121212] after:absolute after:bottom-0 after:left-0 after:right-0 after:h-[1px] after:bg-zinc-800">
                        <tr>
                          {results.columns.map((col) => (
                            <th key={col} className="px-4 py-3 text-xs font-bold text-zinc-400 uppercase tracking-wider bg-zinc-900/80 backdrop-blur-md">
                              {col}
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {paginatedRows.map((row, i) => (
                          <tr key={i} className="border-b border-zinc-800/50 hover:bg-white/[0.02] transition-colors group">
                            {results.columns.map((col) => (
                              <td key={col} className="px-4 py-3 text-sm text-zinc-300 font-mono whitespace-nowrap overflow-hidden text-ellipsis max-w-[300px]">
                                {typeof row[col] === 'object' && row[col] !== null 
                                  ? JSON.stringify(row[col]) 
                                  : String(row[col] ?? 'NULL')}
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  
                  {totalPages > 1 && (
                    <div className="px-4 py-3 border-t border-zinc-800 bg-zinc-900/30 flex items-center justify-between">
                       <span className="text-xs text-zinc-500">
                        Page {currentPage} of {totalPages}
                      </span>
                      <div className="flex gap-2">
                        <button 
                          onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                          disabled={currentPage === 1}
                          className="p-1.5 bg-zinc-800 hover:bg-zinc-700 disabled:opacity-30 rounded-lg text-zinc-400"
                        >
                          <ChevronLeft className="w-4 h-4" />
                        </button>
                        <button 
                          onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                          disabled={currentPage === totalPages}
                          className="p-1.5 bg-zinc-800 hover:bg-zinc-700 disabled:opacity-30 rounded-lg text-zinc-400"
                        >
                          <ChevronRight className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {!results && !error && !loading && (
                <div className="flex flex-col items-center justify-center h-full text-zinc-600 gap-4 anima-fade-in">
                  <div className="p-6 bg-zinc-900/50 rounded-full border border-zinc-800/50">
                    <TableIcon className="w-12 h-12 stroke-[1.5px]" />
                  </div>
                  <div className="text-center">
                    <p className="text-base font-medium text-zinc-400">Ready to query</p>
                    <p className="text-sm">Write your SQL above and hit 'Run Query'</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Sidebar Schema / Info */}
        <div className="hidden lg:flex flex-col bg-[#121212] border border-zinc-800 rounded-2xl overflow-hidden shadow-2xl">
          <div className="p-4 border-b border-zinc-800 bg-zinc-900/50 space-y-3">
            <h3 className="text-sm font-bold flex items-center gap-2">
              <Info className="w-4 h-4 text-blue-400" />
              Database Catalog
            </h3>
            <div className="relative group">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-zinc-500 group-focus-within:text-blue-400 transition-colors" />
              <input 
                type="text"
                placeholder="Search tables or columns..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg py-1.5 pl-9 pr-3 text-xs outline-none focus:border-blue-500/50 transition-all placeholder:text-zinc-600"
              />
            </div>
          </div>
          
          <div className="flex-1 overflow-auto custom-scrollbar p-2 space-y-1">
            {schemaLoading ? (
              <div className="flex flex-col items-center justify-center p-8 gap-3 opacity-50">
                <Loader2 className="w-6 h-6 animate-spin text-zinc-500" />
                <span className="text-[10px] uppercase font-bold tracking-tighter text-zinc-600">Loading catalog...</span>
              </div>
            ) : filteredSchema.length > 0 ? (
              filteredSchema.map((table) => (
                <details key={table.table_name} className="group overflow-hidden rounded-lg">
                  <summary className="flex items-center justify-between p-2 hover:bg-zinc-800 cursor-pointer transition-colors group-open:bg-blue-500/5 group-open:text-blue-400">
                    <div className="flex items-center gap-2">
                      <div className="p-1 rounded bg-blue-500/10 text-blue-400 group-open:bg-blue-500 group-open:text-white transition-all">
                        <TableIcon className="w-3 h-3" />
                      </div>
                      <span className="text-xs font-semibold">{table.table_name}</span>
                    </div>
                    <ChevronRight className="w-3 h-3 text-zinc-600 transition-transform group-open:rotate-90" />
                  </summary>
                  <div className="p-2 pl-4 border-l border-zinc-800 ml-5 space-y-1.5 pb-3">
                    {table.columns.map(col => (
                      <div key={col.name} className="flex flex-col gap-0.5 group/col">
                        <span className="text-[11px] font-mono text-zinc-300 group-hover/col:text-blue-300 transition-colors">{col.name}</span>
                        <span className="text-[9px] font-bold text-zinc-600 uppercase tracking-widest">{col.type}</span>
                      </div>
                    ))}
                  </div>
                </details>
              ))
            ) : (
                <div className="p-8 text-center text-xs text-zinc-600">
                  No matches found for "{searchTerm}"
                </div>
            )}
          </div>

          <div className="p-4 bg-zinc-900/30 border-t border-zinc-800 text-[10px] text-zinc-500 flex flex-col gap-2">
            <p className="font-medium text-zinc-400">Need help?</p>
            <p>Use standard SQL to query system tables. DuckDB docs are available via the 'DuckDB Expert' skill.</p>
            <div className="flex flex-wrap gap-1 mt-1">
              {['sessions', 'messages', 'skills', 'tool_results'].map(t => (
                <button 
                  key={t}
                  onClick={() => setQuery(`SELECT * FROM ${t} LIMIT 5;`)}
                  className="px-2 py-0.5 bg-zinc-800 hover:bg-zinc-700 rounded text-zinc-400 border border-zinc-700/50 transition-colors"
                >
                  {t}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      <style>{`
        .custom-scrollbar::-webkit-scrollbar {
          width: 5px;
          height: 5px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: transparent;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: #333;
          border-radius: 10px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: #444;
        }
        
        @keyframes fade-in {
          from { opacity: 0; transform: translateY(4px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .anima-fade-in {
          animation: fade-in 0.4s ease-out forwards;
        }
      `}</style>
    </div>
  );
};

export default DatabaseExplorer;
