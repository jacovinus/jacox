import { render, screen } from '@testing-library/react';
import ReasoningGraph from './ReasoningGraph';
import { describe, it, expect, vi } from 'vitest';

describe('ReasoningGraph Component', () => {
  const mockHandlers = {
    onNodeMove: vi.fn(),
    onNodeSelect: vi.fn(),
  };

  it('should render the graph container', () => {
    render(
      <ReasoningGraph 
        nodes={{}} 
        edges={[]} 
        {...mockHandlers}
      />
    );
    expect(screen.getByTestId('reasoning-graph-container')).toBeInTheDocument();
  });

  it('should render nodes when provided', () => {
    const nodes = {
      'n1': { id: 'n1', node_type: 'LlmGeneration', payload: {} }
    };
    render(
      <ReasoningGraph 
        nodes={nodes} 
        edges={[]} 
        {...mockHandlers}
      />
    );
    expect(screen.getByText('n1')).toBeInTheDocument();
  });
});
