import { memo, useMemo } from 'react';
import {
    BarChart,
    Bar,
    LineChart,
    Line,
    PieChart,
    Pie,
    Cell,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip as RechartsTooltip,
    ResponsiveContainer,
    Legend
} from 'recharts';

interface ChartData {
    type: 'bar' | 'line' | 'pie';
    title?: string;
    data: any[];
    xAxis?: string;
    yAxis?: string;
    series?: string[];
}

const COLORS = ['#f92672', '#a6e22e', '#66d9ef', '#fd971f', '#ae81ff', '#fabd2f', '#83a598'];

const CustomTooltip = ({ active, payload, xKey }: any) => {
    if (active && payload && payload.length) {
        const item = payload[0].payload;
        return (
            <div className="bg-gruv-dark-1 border border-gruv-dark-4 p-3 rounded-lg shadow-2xl min-w-[120px]">
                <p className="text-[10px] font-bold text-monokai-aqua uppercase tracking-tighter mb-1 border-b border-gruv-dark-4/30 pb-1">
                    {item[xKey]}
                </p>
                {item._subLabel && (
                    <p className="text-[10px] text-gruv-light-4 mb-2 italic">
                        {item._subLabel}
                    </p>
                )}
                {payload.map((p: any, i: number) => (
                    <div key={i} className="flex justify-between items-center gap-4 py-0.5">
                        <span className="text-[11px] text-gruv-light-2" style={{ color: p.color }}>{p.name}:</span>
                        <span className="text-[11px] font-mono font-bold text-gruv-light-1">
                            {typeof p.value === 'number' ? p.value.toLocaleString() : p.value}
                        </span>
                    </div>
                ))}
            </div>
        );
    }
    return null;
};

export const ChartComponent = memo(({ chartData }: { chartData: ChartData }) => {
    const { type, title, data, xAxis, yAxis, series } = chartData;

    const processedData = useMemo(() => {
        if (!data || !Array.isArray(data)) return [];

        const hasNestedValues = data.some(item => Array.isArray(item.values));

        if (hasNestedValues) {
            let globalIdx = 0;
            return data.flatMap((item: any) => {
                if (Array.isArray(item.values)) {
                    return item.values.map((v: any, idx: number) => {
                        const subLabel = Array.isArray(item.subLabels) ? item.subLabels[idx] : null;
                        return {
                            ...item,
                            [yAxis || 'value']: v,
                            _globalIdx: globalIdx++,
                            _subIdx: idx,
                            _subLabel: subLabel,
                            _isFirstInBucket: idx === 0,
                        };
                    });
                }
                return [{ ...item, [yAxis || 'value']: item[yAxis || 'value'] || 0, _isFirstInBucket: true, _globalIdx: globalIdx++ }];
            });
        }

        return data.map((d, i) => ({ ...d, _globalIdx: i }));
    }, [data, yAxis]);

    const renderChart = () => {
        const xKey = xAxis || 'name';
        const yKey = yAxis || 'value';

        switch (type) {
            case 'line':
                return (
                    <LineChart data={processedData}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#3c3836" vertical={false} opacity={0.3} />
                        <XAxis
                            dataKey="_globalIdx"
                            stroke="#a89984"
                            fontSize={9}
                            tickLine={false}
                            axisLine={false}
                            dy={10}
                            interval={0}
                            tick={(props: any) => {
                                const { x, y, index } = props;
                                const item = processedData[index];

                                // Always show the FIRST item's label
                                // Also show at reasonable intervals for very large datasets
                                let shouldShow = item?._isFirstInBucket;
                                if (!shouldShow && processedData.length > 30 && index % Math.floor(processedData.length / 5) === 0) {
                                    shouldShow = true;
                                }

                                if (!shouldShow) return null;

                                return (
                                    <g transform={`translate(${x},${y})`}>
                                        <text
                                            x={0}
                                            y={0}
                                            dy={16}
                                            textAnchor="middle"
                                            fill="#a89984"
                                            fontSize={9}
                                            className="font-medium"
                                        >
                                            {item[xKey]}
                                        </text>
                                    </g>
                                );
                            }}
                        />
                        <YAxis
                            stroke="#a89984"
                            fontSize={9}
                            tickLine={false}
                            axisLine={false}
                            dx={-10}
                            domain={['auto', 'auto']}
                            tickFormatter={(v) => typeof v === 'number' && v >= 1000 ? `${(v / 1000).toFixed(1)}k` : v}
                        />
                        <RechartsTooltip content={<CustomTooltip xKey={xKey} />} cursor={{ stroke: '#504945', strokeWidth: 1 }} />
                        <Legend verticalAlign="top" height={36} wrapperStyle={{ fontSize: '11px', color: '#a89984', paddingBottom: '10px' }} />
                        {series ? series.map((s, i) => (
                            <Line
                                key={s}
                                type="monotone"
                                dataKey={s}
                                stroke={COLORS[i % COLORS.length]}
                                strokeWidth={2}
                                dot={processedData.length < 50}
                                activeDot={{ r: 5, strokeWidth: 0 }}
                                animationDuration={1000}
                                strokeLinecap="round"
                            />
                        )) : (
                            <Line
                                type="monotone"
                                dataKey={yKey}
                                stroke="#66d9ef"
                                strokeWidth={2}
                                dot={processedData.length < 50}
                                activeDot={{ r: 5, strokeWidth: 0 }}
                                animationDuration={1000}
                                strokeLinecap="round"
                            />
                        )}
                    </LineChart>
                );
            case 'pie':
                return (
                    <PieChart>
                        <Pie
                            data={processedData}
                            cx="50%"
                            cy="50%"
                            innerRadius={60}
                            outerRadius={80}
                            paddingAngle={5}
                            dataKey={yKey}
                            nameKey={xKey}
                            animationDuration={1000}
                        >
                            {processedData.map((_, index) => (
                                <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} stroke="none" />
                            ))}
                        </Pie>
                        <RechartsTooltip content={<CustomTooltip xKey={xKey} />} />
                        <Legend verticalAlign="bottom" height={36} wrapperStyle={{ fontSize: '11px', color: '#a89984' }} />
                    </PieChart>
                );
            case 'bar':
            default:
                return (
                    <BarChart data={processedData}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#3c3836" vertical={false} opacity={0.3} />
                        <XAxis
                            dataKey="_globalIdx"
                            stroke="#a89984"
                            fontSize={9}
                            tickLine={false}
                            axisLine={false}
                            dy={10}
                            interval={0}
                            tick={(props: any) => {
                                const { x, y, index } = props;
                                const item = processedData[index];
                                if (!item?._isFirstInBucket && !(processedData.length > 20 && index % Math.floor(processedData.length / 5) === 0)) return null;
                                return (
                                    <g transform={`translate(${x},${y})`}>
                                        <text x={0} y={0} dy={16} textAnchor="middle" fill="#a89984" fontSize={9}>
                                            {item[xKey]}
                                        </text>
                                    </g>
                                );
                            }}
                        />
                        <YAxis
                            stroke="#a89984"
                            fontSize={9}
                            tickLine={false}
                            axisLine={false}
                            dx={-10}
                            domain={['auto', 'auto']}
                        />
                        <RechartsTooltip content={<CustomTooltip xKey={xKey} />} cursor={{ fill: '#32302f', opacity: 0.4 }} />
                        <Legend verticalAlign="top" height={36} wrapperStyle={{ fontSize: '11px', color: '#a89984' }} />
                        {series ? series.map((s, i) => (
                            <Bar
                                key={s}
                                dataKey={s}
                                fill={COLORS[i % COLORS.length]}
                                radius={[4, 4, 0, 0]}
                                animationDuration={1000}
                            />
                        )) : (
                            <Bar
                                dataKey={yKey}
                                fill="#f92672"
                                radius={[4, 4, 0, 0]}
                                animationDuration={1000}
                            />
                        )}
                    </BarChart>
                );
        }
    };

    return (
        <div className="my-8 p-6 rounded-2xl border border-gruv-dark-4/30 bg-gruv-dark-2/50 shadow-xl animate-in fade-in slide-in-from-bottom-4 duration-500">
            {title && (
                <h3 className="text-[11px] font-bold text-monokai-aqua mb-6 uppercase border-b border-gruv-dark-4/30 pb-2 flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-monokai-aqua animate-pulse" />
                    {title}
                </h3>
            )}
            <div className="h-[320px] w-full">
                <ResponsiveContainer width="100%" height="100%">
                    {renderChart()}
                </ResponsiveContainer>
            </div>
            <div className="mt-6 flex justify-between items-center text-[9px] uppercase font-bold tracking-widest text-gruv-light-4/40 border-t border-gruv-dark-4/20 pt-4">
                <span className="flex items-center gap-1.5 italic">
                    <span className="w-1 h-1 rounded-full bg-gruv-light-4/40" />
                    Interactive Visualization
                </span>
                <span className="opacity-60">{type.toUpperCase()} ANALYSIS</span>
            </div>
        </div>
    );
});
