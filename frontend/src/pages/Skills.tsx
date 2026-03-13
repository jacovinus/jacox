import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { skillsApi, type Skill, type CreateSkillRequest, type UpdateSkillRequest } from '../api/skills';
import {
    BookOpen, Plus, Clipboard, Pencil, Trash2, X, Check,
    Link, Loader2, Tag, Search, ChevronDown, ChevronUp
} from 'lucide-react';

function parseTags(tags: string): string[] {
    return tags.split(',').map(t => t.trim()).filter(Boolean);
}

// ─── Tag Badge ───────────────────────────────────────────────────────────────

const TagBadge = ({ label }: { label: string }) => (
    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wide bg-monokai-purple/20 text-monokai-purple border border-monokai-purple/30">
        <Tag className="w-2.5 h-2.5" /> {label}
    </span>
);

// ─── Skill Card ──────────────────────────────────────────────────────────────

interface SkillCardProps {
    skill: Skill;
    onEdit: (skill: Skill) => void;
    onDelete: (id: number) => void;
}

const SkillCard = ({ skill, onEdit, onDelete }: SkillCardProps) => {
    const [copied, setCopied] = useState(false);
    const [expanded, setExpanded] = useState(false);

    const handleCopy = () => {
        navigator.clipboard.writeText(skill.content);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const preview = skill.content.slice(0, 180);
    const isLong = skill.content.length > 180;
    const displayContent = expanded ? skill.content : preview;
    const tags = parseTags(skill.tags);

    return (
        <div className="group relative flex flex-col gap-3 p-5 rounded-2xl bg-gruv-dark-2/60 border border-gruv-dark-4/30 hover:border-monokai-pink/40 transition-all duration-300 hover:shadow-[0_0_20px_rgba(249,38,114,0.08)]">
            {/* Header */}
            <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-2 min-w-0">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-monokai-purple/30 to-monokai-pink/20 flex items-center justify-center flex-shrink-0">
                        <BookOpen className="w-4 h-4 text-monokai-purple" />
                    </div>
                    <div className="min-w-0">
                        <h3 className="font-semibold text-gruv-light-1 truncate">{skill.name}</h3>
                        {skill.source_url && (
                            <a href={skill.source_url} target="_blank" rel="noreferrer"
                               className="text-[10px] text-monokai-aqua hover:underline truncate block">
                                {skill.source_url}
                            </a>
                        )}
                    </div>
                </div>
                {/* Actions */}
                <div className="flex gap-1.5 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <button onClick={handleCopy}
                        className="p-1.5 rounded-lg hover:bg-monokai-green/20 text-gruv-light-4 hover:text-monokai-green transition-colors"
                        title="Copy to clipboard">
                        {copied ? <Check className="w-4 h-4" /> : <Clipboard className="w-4 h-4" />}
                    </button>
                    <button onClick={() => onEdit(skill)}
                        className="p-1.5 rounded-lg hover:bg-monokai-yellow/20 text-gruv-light-4 hover:text-monokai-yellow transition-colors"
                        title="Edit skill">
                        <Pencil className="w-4 h-4" />
                    </button>
                    <button onClick={() => onDelete(skill.id)}
                        className="p-1.5 rounded-lg hover:bg-monokai-red/20 text-gruv-light-4 hover:text-monokai-red transition-colors"
                        title="Delete skill">
                        <Trash2 className="w-4 h-4" />
                    </button>
                </div>
            </div>

            {/* Tags */}
            {tags.length > 0 && (
                <div className="flex flex-wrap gap-1">
                    {tags.map(t => <TagBadge key={t} label={t} />)}
                </div>
            )}

            {/* Content preview */}
            <pre className="text-xs text-gruv-light-4 font-mono whitespace-pre-wrap leading-relaxed rounded-lg bg-gruv-dark-0/50 p-3 border border-gruv-dark-4/20">
                {displayContent}{!expanded && isLong && '...'}
            </pre>

            {isLong && (
                <button onClick={() => setExpanded(!expanded)}
                    className="flex items-center gap-1 text-[11px] text-monokai-aqua hover:text-monokai-aqua/80 transition-colors self-start">
                    {expanded ? <><ChevronUp className="w-3 h-3" />Show less</> : <><ChevronDown className="w-3 h-3" />Show more</>}
                </button>
            )}
        </div>
    );
};

// ─── Skill Form Panel ────────────────────────────────────────────────────────

interface SkillFormProps {
    initial?: Skill | null;
    onClose: () => void;
    onSaved: () => void;
}

const SkillForm = ({ initial, onClose, onSaved }: SkillFormProps) => {
    const qc = useQueryClient();
    const isEdit = !!initial;

    const [name, setName] = useState(initial?.name ?? '');
    const [content, setContent] = useState(initial?.content ?? '');
    const [tags, setTags] = useState(initial?.tags ?? '');
    const [importUrl, setImportUrl] = useState('');
    const [showImport, setShowImport] = useState(false);

    const createMut = useMutation({
        mutationFn: (d: CreateSkillRequest) => skillsApi.create(d),
        onSuccess: () => { qc.invalidateQueries({ queryKey: ['skills'] }); onSaved(); },
    });

    const updateMut = useMutation({
        mutationFn: (d: UpdateSkillRequest) => skillsApi.update(initial!.id, d),
        onSuccess: () => { qc.invalidateQueries({ queryKey: ['skills'] }); onSaved(); },
    });

    const fetchMut = useMutation({
        mutationFn: () => skillsApi.fetchUrl({ url: importUrl, name: name || 'Imported Skill', tags }),
        onSuccess: (skill) => {
            setName(skill.name);
            setContent(skill.content);
            setImportUrl('');
            setShowImport(false);
            qc.invalidateQueries({ queryKey: ['skills'] });
            onSaved();
        },
    });

    const isBusy = createMut.isPending || updateMut.isPending || fetchMut.isPending;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (isEdit) {
            updateMut.mutate({ name, content, tags });
        } else {
            createMut.mutate({ name, content, tags });
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
            <div className="w-full max-w-2xl bg-gruv-dark-1 border border-gruv-dark-4/40 rounded-2xl shadow-2xl flex flex-col max-h-[90vh]">
                {/* Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gruv-dark-4/30">
                    <div className="flex items-center gap-3">
                        <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-monokai-pink to-monokai-purple flex items-center justify-center">
                            <BookOpen className="w-5 h-5 text-white" />
                        </div>
                        <h2 className="text-lg font-bold text-gruv-light-1">
                            {isEdit ? 'Edit Skill' : 'New Skill'}
                        </h2>
                    </div>
                    <button onClick={onClose} className="p-2 rounded-lg hover:bg-gruv-dark-3 text-gruv-light-4 hover:text-white transition-colors">
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Form */}
                <form onSubmit={handleSubmit} className="flex flex-col gap-4 p-6 overflow-y-auto flex-grow">
                    {/* Name */}
                    <div className="flex flex-col gap-1.5">
                        <label className="text-xs font-semibold uppercase tracking-wider text-gruv-light-4">Skill Name</label>
                        <input
                            value={name} onChange={e => setName(e.target.value)} required
                            placeholder="e.g. Code Review Checklist"
                            className="w-full bg-gruv-dark-2 border border-gruv-dark-4/30 rounded-xl px-4 py-2.5 text-gruv-light-1 placeholder:text-gruv-dark-4 focus:outline-none focus:border-monokai-pink/60 transition-colors"
                        />
                    </div>

                    {/* Tags */}
                    <div className="flex flex-col gap-1.5">
                        <label className="text-xs font-semibold uppercase tracking-wider text-gruv-light-4">Tags <span className="normal-case font-normal">(comma-separated)</span></label>
                        <input
                            value={tags} onChange={e => setTags(e.target.value)}
                            placeholder="e.g. prompts, review, code"
                            className="w-full bg-gruv-dark-2 border border-gruv-dark-4/30 rounded-xl px-4 py-2.5 text-gruv-light-1 placeholder:text-gruv-dark-4 focus:outline-none focus:border-monokai-pink/60 transition-colors"
                        />
                    </div>

                    {/* Content */}
                    <div className="flex flex-col gap-1.5">
                        <label className="text-xs font-semibold uppercase tracking-wider text-gruv-light-4">Content (Markdown)</label>
                        <textarea
                            value={content} onChange={e => setContent(e.target.value)} required
                            rows={12}
                            placeholder="Paste or type your skill content here..."
                            className="w-full bg-gruv-dark-0 border border-gruv-dark-4/30 rounded-xl px-4 py-3 text-gruv-light-1 font-mono text-sm placeholder:text-gruv-dark-4 focus:outline-none focus:border-monokai-pink/60 transition-colors resize-y"
                        />
                    </div>

                    {/* Import from URL */}
                    {!isEdit && (
                        <div>
                            <button type="button" onClick={() => setShowImport(!showImport)}
                                className="flex items-center gap-2 text-sm text-monokai-aqua hover:text-monokai-aqua/80 transition-colors">
                                <Link className="w-4 h-4" />
                                {showImport ? 'Cancel URL import' : 'Import from URL'}
                            </button>
                            {showImport && (
                                <div className="mt-3 flex gap-2">
                                    <input
                                        value={importUrl} onChange={e => setImportUrl(e.target.value)}
                                        placeholder="https://raw.githubusercontent.com/..."
                                        className="flex-grow bg-gruv-dark-2 border border-gruv-dark-4/30 rounded-xl px-4 py-2.5 text-gruv-light-1 placeholder:text-gruv-dark-4 focus:outline-none focus:border-monokai-aqua/60 transition-colors text-sm"
                                    />
                                    <button type="button" disabled={!importUrl || fetchMut.isPending}
                                        onClick={() => fetchMut.mutate()}
                                        className="px-4 py-2.5 rounded-xl bg-monokai-aqua/20 text-monokai-aqua hover:bg-monokai-aqua/30 disabled:opacity-50 transition-colors font-semibold text-sm flex items-center gap-2">
                                        {fetchMut.isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : 'Fetch & Save'}
                                    </button>
                                </div>
                            )}
                        </div>
                    )}
                </form>

                {/* Footer */}
                <div className="flex justify-end gap-3 px-6 py-4 border-t border-gruv-dark-4/30">
                    <button type="button" onClick={onClose}
                        className="px-5 py-2.5 rounded-xl text-gruv-light-4 hover:text-white hover:bg-gruv-dark-3 transition-colors font-semibold text-sm">
                        Cancel
                    </button>
                    <button
                        onClick={handleSubmit as any}
                        disabled={isBusy || !name || !content}
                        className="px-5 py-2.5 rounded-xl bg-gradient-to-r from-monokai-pink to-monokai-purple text-white font-semibold text-sm shadow-[0_0_15px_rgba(249,38,114,0.3)] hover:shadow-[0_0_25px_rgba(249,38,114,0.5)] disabled:opacity-50 transition-all duration-300 flex items-center gap-2">
                        {isBusy && <Loader2 className="w-4 h-4 animate-spin" />}
                        {isEdit ? 'Save Changes' : 'Create Skill'}
                    </button>
                </div>
            </div>
        </div>
    );
};

// ─── Main Page ───────────────────────────────────────────────────────────────

export const Skills = () => {
    const qc = useQueryClient();
    const [showForm, setShowForm] = useState(false);
    const [editTarget, setEditTarget] = useState<Skill | null>(null);
    const [search, setSearch] = useState('');

    const { data: skills = [], isLoading } = useQuery({
        queryKey: ['skills'],
        queryFn: () => skillsApi.list(),
    });

    const deleteMut = useMutation({
        mutationFn: (id: number) => skillsApi.delete(id),
        onSuccess: () => qc.invalidateQueries({ queryKey: ['skills'] }),
    });

    const filtered = skills.filter(s => {
        const q = search.toLowerCase();
        return !q || s.name.toLowerCase().includes(q) || s.content.toLowerCase().includes(q) || s.tags.toLowerCase().includes(q);
    });

    const handleEdit = (skill: Skill) => {
        setEditTarget(skill);
        setShowForm(true);
    };

    const handleClose = () => {
        setShowForm(false);
        setEditTarget(null);
    };

    return (
        <div className="space-y-8">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-gruv-light-4 bg-clip-text text-transparent">
                        Skills
                    </h1>
                    <p className="text-gruv-light-4 mt-1 text-sm">
                        Reusable markdown snippets — system prompts, templates, context blocks.
                    </p>
                </div>
                <button
                    onClick={() => { setEditTarget(null); setShowForm(true); }}
                    className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-monokai-pink to-monokai-purple text-white font-semibold shadow-[0_0_15px_rgba(249,38,114,0.3)] hover:shadow-[0_0_25px_rgba(249,38,114,0.5)] transition-all duration-300">
                    <Plus className="w-4 h-4" /> New Skill
                </button>
            </div>

            {/* Search */}
            <div className="relative">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-gruv-dark-4" />
                <input
                    value={search} onChange={e => setSearch(e.target.value)}
                    placeholder="Search skills by name, content, or tags..."
                    className="w-full bg-gruv-dark-2/60 border border-gruv-dark-4/30 rounded-xl pl-11 pr-4 py-3 text-gruv-light-1 placeholder:text-gruv-dark-4 focus:outline-none focus:border-monokai-pink/60 transition-colors"
                />
            </div>

            {/* Stats */}
            <div className="flex gap-4">
                <div className="px-4 py-2 rounded-xl bg-gruv-dark-2/40 border border-gruv-dark-4/20 text-sm">
                    <span className="text-monokai-pink font-bold">{skills.length}</span>
                    <span className="text-gruv-light-4 ml-1">total skills</span>
                </div>
                {search && (
                    <div className="px-4 py-2 rounded-xl bg-gruv-dark-2/40 border border-gruv-dark-4/20 text-sm">
                        <span className="text-monokai-aqua font-bold">{filtered.length}</span>
                        <span className="text-gruv-light-4 ml-1">matching</span>
                    </div>
                )}
            </div>

            {/* Grid */}
            {isLoading ? (
                <div className="flex items-center justify-center py-24">
                    <Loader2 className="w-8 h-8 animate-spin text-monokai-pink" />
                </div>
            ) : filtered.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-24 gap-4">
                    <div className="w-16 h-16 rounded-2xl bg-gruv-dark-2/60 flex items-center justify-center">
                        <BookOpen className="w-8 h-8 text-gruv-dark-4" />
                    </div>
                    <p className="text-gruv-light-4 font-medium">
                        {search ? 'No skills match your search.' : 'No skills yet. Create your first one!'}
                    </p>
                    {!search && (
                        <button onClick={() => { setEditTarget(null); setShowForm(true); }}
                            className="flex items-center gap-2 px-4 py-2 rounded-xl border border-monokai-pink/40 text-monokai-pink hover:bg-monokai-pink/10 transition-colors text-sm font-semibold">
                            <Plus className="w-4 h-4" /> Create Skill
                        </button>
                    )}
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {filtered.map(skill => (
                        <SkillCard
                            key={skill.id}
                            skill={skill}
                            onEdit={handleEdit}
                            onDelete={id => deleteMut.mutate(id)}
                        />
                    ))}
                </div>
            )}

            {/* Modal */}
            {showForm && (
                <SkillForm
                    initial={editTarget}
                    onClose={handleClose}
                    onSaved={handleClose}
                />
            )}
        </div>
    );
};
