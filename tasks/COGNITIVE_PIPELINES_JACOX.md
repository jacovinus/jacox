Integrate the Cognitive Pipelines from `jac_llmos` into the `jacox` orchestrator and provide a user interface for building, managing, and executing them.

> [!IMPORTANT]
> **Optional Plug-and-Play**: `jac_llmos` must remain an optional component. The system must detect its presence (via health check) and only enable pipeline features if it is reachable. No part of `jacox` should block or error out if `jac_llmos` is offline.

## 1. Backend Integration (Jacox)

**Objective**: Provide persistence and execution orchestration for pipelines.

- **Database Persistence**:
  - [ ] Define `pipelines` table in `chat.db` (id, name, definition_json, created_at).
  - [ ] Implement CRUD operations in the database layer.
- **Connectivity & Discovery**:
  - [ ] Implement a `LlmosStatus` service to poll the health of `jac_llmos`.
  - [ ] Add a `GET /api/llmos/status` endpoint for the frontend.
- **API Endpoints**:
  - [ ] ...
  - [ ] `POST /api/pipelines/execute/:id`: ... Return a clear `503 Service Unavailable` if `jac_llmos` is disconnected.
- **Security**:
  - [ ] Ensure all pipeline endpoints are protected by the rolling token middleware.

## 2. Frontend Integration

**Objective**: A high-end, premium UI for interacting with cognitive pipelines.

- **Global State / Context**:
  - [ ] Implement a `useLlmos` hook to track connectivity status.
- **Components**:
  - [ ] `Sidebar`: Hide the "Pipelines" item if `jac_llmos` is not detected.
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

### Backend Tests (`jacox/tests/pipeline_integration.rs`)
- [ ] Test DB persistence (Save/Load/Delete).
- [ ] Test API orchestration with mock `jac_llmos` responses.
- [ ] Test error handling (invalid JSON, missing stages).

### Frontend Tests (`jacox/frontend/src/pages/Pipelines.test.tsx`)
- [ ] Test component rendering with mock data.
- [ ] Test execution flow: sending request and displaying received trace.
- [ ] Test editor validation.

## 4. Deliverables
1. Functional Pipeline CRUD in Jacox.
2. Executable pipelines via the Jacox UI.
3. Visual reasoning trace for every execution.
4. Comprehensive documentation update.
