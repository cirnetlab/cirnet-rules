export type SaveState = { loading: boolean };
type SaveEvent = (name: 'saved' | 'failed', payload: string) => void;

export async function savePanel(
  state: SaveState,
  graphId: string,
  saveGraph: (id: string) => Promise<void>,
  emit: SaveEvent,
): Promise<void> {
  if (state.loading) return;
  state.loading = true;
  try {
    await saveGraph(graphId);
    emit('saved', graphId);
    state.loading = false;
  } catch (error) {
    emit('failed', error instanceof Error ? error.message : String(error));
  }
}
