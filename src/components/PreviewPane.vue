<template>
  <div class="flex h-full flex-col">
    <div class="mb-3 border-b border-border/80 pb-3">
      <p class="section-eyebrow">Preview</p>
      <h3 class="section-title">{{ $t('preview.title') }}</h3>
    </div>

    <div
      class="flex flex-1 items-center justify-center overflow-hidden border border-border/80 bg-[#151518] p-4"
    >
      <div v-if="previewUri && isImage" class="flex h-full w-full flex-col items-center">
        <el-image
          :src="previewUri"
          :alt="fileName || 'Preview'"
          class="preview-image"
          fit="scale-down"
          :zoom-rate="1.2"
          :max-scale="7"
          :min-scale="0.2"
          :preview-src-list="[previewUri]"
          :initial-index="0"
          show-progress
        >
          <template #error>
            <div class="empty-state min-h-full border-0 bg-transparent">
              <CircleAlert class="size-12 text-destructive" />
              <p class="text-sm font-medium text-destructive">
                {{ $t('preview.previewLoadFailed') }}
              </p>
            </div>
          </template>
        </el-image>
        <div v-if="fileName" class="mt-3 break-all text-center text-sm text-muted-foreground">
          {{ fileName }}
        </div>
      </div>

      <div
        v-else-if="previewUri && !isImage"
        class="empty-state min-h-full w-full border-0 bg-transparent"
      >
        <File class="size-12 text-muted-foreground" />
        <p class="text-sm font-medium text-foreground">{{ $t('preview.unsupportedFileType') }}</p>
        <p v-if="fileName" class="break-all text-sm text-muted-foreground">{{ fileName }}</p>
      </div>

      <div v-else class="empty-state min-h-full w-full border-0 bg-transparent">
        <FileText class="size-12 text-muted-foreground" />
        <p class="text-sm font-medium text-foreground">{{ $t('preview.selectFileToPreview') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CircleAlert, File, FileText } from 'lucide-vue-next'

interface Props {
  previewUri?: string
  fileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  previewUri: '',
  fileName: ''
})

const isImage = computed(() => {
  if (!props.previewUri) return false
  const ext = props.previewUri.split('.').pop()?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg'].includes(ext || '')
})
</script>

<style scoped>
.preview-image {
  height: min(60vh, 40rem);
  width: 100%;
}

.preview-image :deep(.el-image__inner) {
  width: 100%;
  height: 100%;
  object-fit: scale-down;
}

.preview-image :deep(.el-image__error) {
  background: transparent;
}
</style>
