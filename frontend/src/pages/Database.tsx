import { useQuery } from '@tanstack/react-query';
import { Database as DbIcon, Search, ArrowRight, Table } from 'lucide-react';
import { sessionsApi } from '../api/sessions';
import type { Session } from '../types';

export const Database = () => {
    const { data: sessions, isLoading } = useQuery({
        queryKey: ['sessions'],
        queryFn: () => sessionsApi.list()
    });

    return (
        <div className="flex flex-col gap-8">
            <header className="flex justify-between items-end">
                <div>
                    <h1 className="text-4xl font-bold flex items-center gap-3">
                        <DbIcon className="text-monokai-aqua" />
                        DuckDB Explorer
                    </h1>
                    <p className="text-gruv-light-4">Direct access to conversation storage.</p>
                </div>
                <div className="flex gap-2">
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gruv-light-4" />
                        <input
                            type="text"
                            placeholder="Query sessions..."
                            className="bg-gruv-dark-3 border border-gruv-dark-4 rounded-lg py-2 pl-10 pr-4 text-sm focus:outline-none focus:border-monokai-aqua"
                        />
                    </div>
                </div>
            </header>

            <div className="glass rounded-2xl overflow-hidden">
                <div className="bg-gruv-dark-2/50 p-4 border-b border-gruv-dark-4/20 flex items-center gap-2">
                    <Table className="w-4 h-4 text-monokai-pink" />
                    <span className="font-mono text-sm font-bold uppercase tracking-widest text-gruv-light-4">Table: sessions</span>
                </div>
                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-gruv-dark-0/30 text-gruv-light-4 text-xs font-mono uppercase">
                                <th className="p-4 border-b border-gruv-dark-4/10">ID</th>
                                <th className="p-4 border-b border-gruv-dark-4/10">Name</th>
                                <th className="p-4 border-b border-gruv-dark-4/10">Created At</th>
                                <th className="p-4 border-b border-gruv-dark-4/10">Metadata</th>
                                <th className="p-4 border-b border-gruv-dark-4/10">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="text-sm font-mono">
                            {isLoading ? (
                                <tr>
                                    <td colSpan={5} className="p-8 text-center text-gruv-light-4">Loading data...</td>
                                </tr>
                            ) : (
                                sessions?.map((s: Session) => (
                                    <tr key={s.id} className="hover:bg-gruv-dark-4/10 transition-colors group">
                                        <td className="p-4 border-b border-gruv-dark-4/10 text-monokai-aqua truncate max-w-[120px]">{s.id}</td>
                                        <td className="p-4 border-b border-gruv-dark-4/10 text-gruv-light-2 font-semibold font-sans">{s.name}</td>
                                        <td className="p-4 border-b border-gruv-dark-4/10 text-gruv-light-4">{new Date(s.created_at).toLocaleString()}</td>
                                        <td className="p-4 border-b border-gruv-dark-4/10 text-monokai-purple truncate max-w-[150px]">
                                            {JSON.stringify(s.metadata)}
                                        </td>
                                        <td className="p-4 border-b border-gruv-dark-4/10">
                                            <button className="text-monokai-pink hover:underline flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                                View Messages <ArrowRight className="w-3 h-3" />
                                            </button>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
};
