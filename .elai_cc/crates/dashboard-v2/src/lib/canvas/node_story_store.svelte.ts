// node_story_store.svelte.ts — open/close state for the NEW DIFF node-story
// overlay. The story panel (NodeStoryPanel.svelte) is mounted once in
// CanvasWindow as an overlay; any canvas surface (PlanMode node, SystemMode
// node inspector) opens it by setting the selected node id here.
//
// Backend pending: openFor(nodeId) currently just opens the mock fixture
// (node_story.ts NODE_STORY_FIXTURE). When the SSB/DKG read-command exists,
// this store fetches the real NodeStory for the id; the consumers stay the same.

function createNodeStoryStore() {
  let open = $state<boolean>(false);
  let nodeId = $state<string | null>(null);

  return {
    get open() { return open; },
    get nodeId() { return nodeId; },
    /** Open the node-story overlay for a given node id. */
    openFor(id: string) {
      nodeId = id;
      open = true;
    },
    close() {
      open = false;
    },
  };
}

export const nodeStory = createNodeStoryStore();
