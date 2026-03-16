Integrate the Cognitive Pipelines from `stepbit-core` into the `stepbit` orchestrator and provide a user interface for building, managing, and executing them.

> [!IMPORTANT]
> **Optional Plug-and-Play**: `stepbit-core` must remain an optional component. The system must detect its presence (via health check) and only enable pipeline features if it is reachable. No part of `stepbit` should block or error out if `stepbit-core` is offline.

## 1. Backend Integration (Stepbit)

**Objective**: Provide persistence and execution orchestration for pipelines.

- **Database Persistence**:
  - [ ] Define `pipelines` table in `chat.db` (id, name, definition_json, created_at).
  - [ ] Implement CRUD operations in the database layer.
- **Connectivity & Discovery**:
  - [ ] Implement a `LlmosStatus` service to poll the health of `stepbit-core`.
  - [ ] Add a `GET /api/llmos/status` endpoint for the frontend.
- **API Endpoints**:
  - [ ] ...
  - [ ] `POST /api/pipelines/execute/:id`: ... Return a clear `503 Service Unavailable` if `stepbit-core` is disconnected.
- **Security**:
  - [ ] Ensure all pipeline endpoints are protected by the rolling token middleware.

## 2. Frontend Integration

**Objective**: A high-end, premium UI for interacting with cognitive pipelines.

- **Global State / Context**:
  - [ ] Implement a `useLlmos` hook to track connectivity status.
- **Components**:
  - [ ] `Sidebar`: Hide the "Pipelines" item if `stepbit-core` is not detected.
  - [ ] `Page Wrappers`: Show a "Disconnected" notice or redirect if a user manually navigates to a pipeline page while the service is down.
- **Components**:
  - [ ] `PipelineList`: Card-based view of available pipelines with quick-action buttons (Run, Edit, Delete).
  - [ ] `PipelineEditor`: A premium JSON/YAML editor (Monaco or similar) with real-time validation against the pipeline schema.
  - [ ] `PipelineTraceViewer`: A visual timeline showing each stage of the executed pipeline, its status, and its trace entry.
  - [ ] `PipelineResultPreview`: Structured view for intermediate results (tables for DuckDB data, text blocks for LLM).
- **Pages**:
  - [ ] `PipelinesPage`: Main hub for pipeline management.
  - [ ] `PipelineExecutionPage`: Side-by-side view of settings and real-time execution trace.
- **Navigation**:
  - [ ] Add "Pipelines" to the main sidebar with a modern icon.

## 3. TDD & Verification Plan

**Strict TDD approach**: Write tests before implementation for both backend and frontend.

### Backend Tests (`stepbit/tests/pipeline_integration.rs`)
- [ ] Test DB persistence (Save/Load/Delete).
- [ ] Test API orchestration with mock `stepbit-core` responses.
- [ ] Test error handling (invalid JSON, missing stages).

### Frontend Tests (`stepbit/frontend/src/pages/Pipelines.test.tsx`)
- [ ] Test component rendering with mock data.
- [ ] Test execution flow: sending request and displaying received trace.
- [ ] Test editor validation.

## 4. Deliverables
1. Functional Pipeline CRUD in Stepbit.
2. Executable pipelines via the Stepbit UI.
3. Visual reasoning trace for every execution.
4. Comprehensive documentation update.
