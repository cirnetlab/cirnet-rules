<script setup lang="ts">
import { computed, reactive } from 'vue';
import GraphPreview from './GraphPreview.vue';
import { savePanel } from './save-panel.ts';

const props = defineProps<{
  graphId: string;
  selectedNode: string | null;
  saveGraph: (id: string) => Promise<void>;
}>();
const emit = defineEmits<{
  saved: [id: string];
  failed: [message: string];
}>();

const state = reactive({ loading: false });
const highlight = computed(() => props.selectedNode?.toLowerCase() ?? '');
function save() {
  return savePanel(state, props.graphId, props.saveGraph, emit);
}
</script>

<template>
  <GraphPreview :graph-id="graphId" :highlight="highlight" />
  <button :disabled="state.loading" @click="save">
    {{ state.loading ? 'Saving…' : 'Save graph' }}
  </button>
</template>

<style scoped>
button {
  margin-block-start: 0.75rem;
  color: #274358;
}
</style>
