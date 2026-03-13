import { render, screen } from '@testing-library/react';
import ReasoningGraph from './ReasoningGraph';
import { describe, it, expect } from 'vitest';

describe('ReasoningGraph Component', () => {
  it('should render the graph container', () => {
    const mockGraph = { nodes: {}, edges: [] };
    render(<ReasoningGraph graph={mockGraph} />);
    expect(screen.getByTestId('reasoning-graph-container')).toBeInTheDocument();
  });

  it('should render nodes when provided', () => {
    const mockGraph = {
      nodes: {
        'n1': { id: 'n1', node_type: 'LlmGeneration', payload: {} }
      },
      edges: []
    };
    render(<ReasoningGraph graph={mockGraph} />);
    expect(screen.getByText('n1')).toBeInTheDocument();
  });
});
