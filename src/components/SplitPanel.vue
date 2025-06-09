<script setup lang="ts">
import { ref, onUnmounted } from 'vue'

const props = defineProps({
  leftWidth: {
    type: Number,
    default: 7
  },
  rightWidth: {
    type: Number,
    default: 3
  }
})

const emit = defineEmits(['update:leftWidth', 'update:rightWidth'])

const isDragging = ref(false)
const startX = ref(0)
const startLeftWidth = ref(0)
const splitLayout = ref<HTMLElement | null>(null)

function startDrag(e: MouseEvent) {
  isDragging.value = true
  startX.value = e.clientX
  startLeftWidth.value = props.leftWidth
  document.addEventListener('mousemove', handleDrag)
  document.addEventListener('mouseup', stopDrag)
}

function handleDrag(e: MouseEvent) {
  if (!isDragging.value || !splitLayout.value) return
  
  const containerWidth = splitLayout.value.offsetWidth
  const deltaX = e.clientX - startX.value
  const newLeftWidth = startLeftWidth.value + (deltaX / containerWidth) * 10
  
  // Limit min/max width
  if (newLeftWidth > 1 && newLeftWidth < 9) {
    emit('update:leftWidth', newLeftWidth)
    emit('update:rightWidth', 10 - newLeftWidth)
  }
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', handleDrag)
  document.removeEventListener('mouseup', stopDrag)
}

onUnmounted(() => {
  document.removeEventListener('mousemove', handleDrag)
  document.removeEventListener('mouseup', stopDrag)
})
</script>

<template>
  <div class="split-layout" ref="splitLayout">
    <div class="left-panel">
      <slot name="left"></slot>
    </div>
    <div class="split-handle" @mousedown="startDrag"></div>
    <div class="right-panel">
      <slot name="right"></slot>
    </div>
  </div>
</template>

<style scoped lang="scss">
.split-layout {
  display: flex;
  gap: 0;
  height: 100%;
  position: relative;
  
  .left-panel {
    flex: v-bind('props.leftWidth');
    overflow: hidden;
  }
  
  .split-handle {
    width: 8px;
    cursor: col-resize;
    transition: background-color 0.2s;
    
    &:hover {
      background-color: #ddd;
    }
  }
  
  .right-panel {
    flex: v-bind('props.rightWidth');
    overflow-y: auto;
    border-left: 1px solid #ddd;
    padding-left: 16px;
  }
}
</style>