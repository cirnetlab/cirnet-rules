<script setup lang="ts">
defineProps<{ graphId: string; highlight: string }>();
</script>

<template>
  <output :data-graph="graphId">{{ highlight }}</output>
</template>
