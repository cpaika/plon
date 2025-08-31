# E2E Test Report: Drag and Drop Functionality

## Test Overview
Comprehensive end-to-end tests have been created to validate the drag and drop functionality in both the Kanban board and Map view. These tests will help identify and document bugs in the current implementation.

## Test Suites

### 1. Kanban Board Drag and Drop Tests (`kanban-drag-drop.spec.ts`)

#### Tests Implemented:
- ✅ Display kanban board with all columns
- ✅ Display task cards with draggable attribute
- ✅ Drag task from Todo to In Progress
- ✅ Show visual feedback when dragging over column
- ✅ Handle drag and drop between non-adjacent columns
- ✅ Handle dragging to empty column
- ✅ Delete task when clicking X button
- ✅ Maintain task properties after drag
- ✅ Handle rapid consecutive drags
- ✅ Handle drag cancellation (ESC key)
- ✅ Handle multiple cards in same column

#### Expected Behaviors Tested:
1. **Column highlighting**: Columns should change background color when a card is dragged over them
2. **Task count updates**: Column counts should update immediately after drag and drop
3. **Task persistence**: Task properties (title, description, priority) should remain unchanged after dragging
4. **Empty column handling**: "Drop tasks here" placeholder should appear/disappear appropriately
5. **Drag cancellation**: ESC key should cancel an in-progress drag operation

### 2. Map View Drag Tests (`map-drag.spec.ts`)

#### Tests Implemented:
- ✅ Display map view with task cards
- ✅ Have draggable task cards with position styles
- ✅ Drag task card to new position
- ✅ Show visual feedback when dragging
- ✅ Handle zoom controls
- ✅ Scale cards according to zoom level
- ✅ Add new task with Add Task button
- ✅ Change task status with status button
- ✅ Delete task with delete button
- ✅ Select task and show details panel
- ✅ Handle drag with zoom applied
- ✅ Handle multiple tasks dragging
- ✅ Check for task overlap after dragging

#### Expected Behaviors Tested:
1. **Position updates**: Task positions should update to drop location
2. **Visual feedback**: Cards should show opacity change when being dragged
3. **Zoom interaction**: Dragging should work correctly at different zoom levels
4. **Task selection**: Clicking a task should show selection border and details panel
5. **Position persistence**: Tasks should maintain their new positions after dragging

## Known Issues to Test For

Based on the implementation, these are potential bugs the tests will uncover:

### Kanban Board Issues:
1. **Drag state persistence**: `dragging_task` signal might not clear properly on failed drags
2. **Column highlight stuck**: `drag_over_status` might remain set if mouse leaves quickly
3. **Task duplication**: Potential race condition when dragging quickly between columns
4. **Visual feedback delay**: Opacity changes might not trigger immediately
5. **Event propagation**: Click events might interfere with drag events

### Map View Issues:
1. **Simplified position calculation**: Currently moves tasks by fixed offset (50px) instead of actual drop position
2. **Zoom scaling issues**: Mouse position might not be correctly calculated with zoom applied
3. **No collision detection**: Tasks can overlap completely
4. **Drag offset not tracked**: Cards jump to cursor position instead of maintaining drag offset
5. **Performance with many tasks**: Dragging might be laggy with many tasks on screen

## Running the Tests

### Prerequisites:
```bash
npm install
cargo build --bin plon-web
```

### Run all tests:
```bash
./run-e2e-tests.sh
```

### Run specific test suite:
```bash
# Kanban tests only
npx playwright test e2e-tests/kanban-drag-drop.spec.ts

# Map view tests only
npx playwright test e2e-tests/map-drag.spec.ts
```

### Debug mode with browser visible:
```bash
npx playwright test --debug
```

### View test report:
```bash
npx playwright show-report
```

## Test Helpers

The `test-helpers.ts` file provides reusable utilities:
- Navigation helpers
- Element selectors
- Drag and drop simulation
- Task counting and verification
- Screenshot capture for debugging

## Expected Test Results

### Likely to Pass:
- Basic display tests
- Simple single drag operations
- Button click operations (delete, status change)
- Zoom controls

### Likely to Fail or Be Flaky:
- Complex drag feedback (opacity, cursor changes)
- Precise position calculations in map view
- Rapid consecutive drags
- Drag cancellation with ESC
- Drag operations with zoom applied

## Recommendations for Fixes

Based on the test implementation, here are recommended fixes:

1. **Kanban Board**:
   - Implement proper `ondragenter` and `ondragleave` event handling
   - Add debouncing for drag state updates
   - Ensure atomic state updates for task status changes

2. **Map View**:
   - Track actual mouse position during drag
   - Calculate drop position relative to container and zoom level
   - Add collision detection or snapping to grid
   - Implement proper drag offset tracking

3. **Both Views**:
   - Add loading states during drag operations
   - Implement optimistic UI updates with rollback on failure
   - Add proper error handling for failed drag operations

## Next Steps

1. Run the tests to identify actual failures
2. Document specific bug behaviors with screenshots
3. Prioritize fixes based on user impact
4. Implement fixes iteratively with test validation
5. Add more edge case tests as bugs are discovered