import { memo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { xonokai } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { ChartComponent } from './ChartComponent';

interface MarkdownContentProps {
    content: string;
}

const MarkdownComponents: any = {
    code({ inline, className, children, ...props }: any) {
        const match = /language-(\w+)/.exec(className || '');
        const lang = match?.[1] || '';
        const isSvg = lang === 'svg';
        const isCss = lang === 'css';
        const isJson = lang === 'json' || lang === 'chart';

        if (!inline && isJson) {
            try {
                const rawContent = String(children || '').trim();
                if (!rawContent) return null;

                const cleanContent = rawContent.replace(/\/\/.*$/gm, '');
                const data = JSON.parse(cleanContent);
                if (data.role === 'chart' || lang === 'chart') {
                    return <ChartComponent chartData={data} />;
                }
            } catch (e) {
                // Fallback
            }
        }

        if (!inline && isSvg) {
            return (
                <div className="my-6 p-8 rounded-2xl border border-gruv-dark-4/30 bg-gruv-dark-2/50 flex flex-col items-center gap-4 group">
                    <div
                        className="max-w-full overflow-hidden transition-transform duration-500 group-hover:scale-105"
                        dangerouslySetInnerHTML={{ __html: String(children) }}
                    />
                    <div className="text-[10px] uppercase font-bold tracking-widest text-gruv-light-4/40 mt-2 border-t border-gruv-dark-4/20 pt-2 px-4 text-center">
                        Live SVG Graphic
                    </div>
                </div>
            );
        }

        if (!inline && isCss) {
            const cssContent = String(children).replace(/\n$/, '');
            return (
                <div className="my-4 rounded-xl overflow-hidden border border-gruv-dark-4/30 shadow-2xl bg-gruv-dark-3/30">
                    <style dangerouslySetInnerHTML={{ __html: cssContent }} />
                    <div className="bg-gruv-dark-3 px-4 py-2 text-[10px] font-mono text-monokai-aqua flex justify-between items-center border-b border-gruv-dark-4/20">
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full bg-monokai-aqua animate-pulse" />
                            <span>CSS LIVE STYLE</span>
                        </div>
                    </div>
                    <SyntaxHighlighter
                        language="css"
                        style={xonokai}
                        customStyle={{
                            margin: 0,
                            padding: '1.5rem',
                            fontSize: '0.85rem',
                            background: 'transparent',
                        }}
                        {...props}
                    >
                        {cssContent}
                    </SyntaxHighlighter>
                </div>
            );
        }

        return !inline && match ? (
            <div className="my-4 rounded-xl overflow-hidden border border-gruv-dark-4/30 shadow-2xl">
                <div className="bg-gruv-dark-3 px-4 py-2 text-[10px] font-mono text-gruv-light-4 flex justify-between items-center border-b border-gruv-dark-4/20">
                    <span>{match[1].toUpperCase()}</span>
                </div>
                <SyntaxHighlighter
                    language={match[1]}
                    style={xonokai}
                    customStyle={{
                        margin: 0,
                        padding: '1.5rem',
                        fontSize: '0.85rem',
                        background: 'transparent',
                    }}
                    {...props}
                >
                    {String(children).replace(/\n$/, '')}
                </SyntaxHighlighter>
            </div>
        ) : (
            <code className="bg-gruv-dark-4 px-1.5 py-0.5 rounded text-monokai-aqua font-mono text-[0.9em]" {...props}>
                {children}
            </code>
        );
    },
    table({ children }: any) {
        return (
            <div className="my-6 overflow-x-auto rounded-xl border border-gruv-dark-4/30">
                <table className="w-full text-sm text-left border-collapse">
                    {children}
                </table>
            </div>
        );
    },
    thead({ children }: any) {
        return <thead className="bg-gruv-dark-3 text-monokai-aqua font-bold border-b border-gruv-dark-4/30">{children}</thead>;
    },
    th({ children }: any) {
        return <th className="px-4 py-3 border-r border-gruv-dark-4/30 last:border-0">{children}</th>;
    },
    td({ children }: any) {
        return <td className="px-4 py-3 border-b border-r border-gruv-dark-4/30 last:border-0">{children}</td>;
    },
    tr({ children }: any) {
        return <tr className="hover:bg-gruv-dark-3/30 transition-colors last:border-0">{children}</tr>;
    },
    p({ children }: any) {
        return <div className="mb-4 last:mb-0 leading-relaxed">{children}</div>;
    },
    div({ children, className, ...props }: any) {
        // Special container for live demos
        if (className?.includes('live-playground')) {
            return (
                <div className="my-6 p-10 rounded-3xl border border-dashed border-monokai-pink/30 bg-monokai-pink/5 flex flex-col items-center justify-center gap-4 relative overflow-hidden" {...props}>
                    <div className="absolute top-2 right-4 text-[8px] font-mono text-monokai-pink opacity-40 uppercase tracking-[0.2em]">Playground Area</div>
                    {children}
                </div>
            );
        }
        return <div className={className} {...props}>{children}</div>;
    },
    a({ children, href }: any) {
        return <a href={href} className="text-monokai-aqua hover:underline underline-offset-4" target="_blank" rel="noopener noreferrer">{children}</a>;
    },
    ul({ children }: any) {
        return <ul className="list-disc ml-6 mb-4 flex flex-col gap-2">{children}</ul>;
    },
    ol({ children }: any) {
        return <ol className="list-decimal ml-6 mb-4 flex flex-col gap-2">{children}</ol>;
    },
    h1: ({ children }: any) => <h1 className="text-2xl font-bold mb-4 text-monokai-aqua">{children}</h1>,
    h2: ({ children }: any) => <h2 className="text-xl font-bold mb-3 text-monokai-aqua">{children}</h2>,
    h3: ({ children }: any) => <h3 className="text-lg font-bold mb-2 text-monokai-aqua">{children}</h3>,
    blockquote: ({ children }: any) => (
        <blockquote className="border-l-4 border-monokai-purple bg-monokai-purple/5 px-6 py-4 my-6 italic text-gruv-light-2 rounded-r-xl">
            {children}
        </blockquote>
    ),
    img: ({ src, alt }: any) => (
        <div className="my-6 rounded-2xl overflow-hidden border border-gruv-dark-4/30 shadow-2xl bg-gruv-dark-2/50 group relative max-w-2xl mx-auto">
            <img
                src={src}
                alt={alt}
                className="max-w-full h-auto block mx-auto transition-all duration-500 group-hover:scale-[1.01]"
                loading="lazy"
            />
            {alt && (
                <div className="absolute bottom-0 left-0 right-0 p-3 bg-gradient-to-t from-black/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
                    <p className="text-xs text-gruv-light-2 text-center">{alt}</p>
                </div>
            )}
        </div>
    ),
};

export const MarkdownContent = memo(({ content }: MarkdownContentProps) => {
    return (
        <div className="prose prose-invert max-w-none break-words">
            <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                rehypePlugins={[rehypeRaw]}
                components={MarkdownComponents}
            >
                {content}
            </ReactMarkdown>
        </div>
    );
});
