<script setup lang="ts">
import { ref } from 'vue'

const selectedValue = defineModel<string>()

const showMenu = ref(true)
const leftPanelWidth = ref(6)
const rightPanelWidth = ref(4)

const items = [
  { identifier: 'MHWs_STM_Release', sourceType: 'local', filePath: '', hasUpdate: false },
  { identifier: 'MHWs_STM_Release_2', sourceType: 'remote', filePath: '', hasUpdate: true }
]

const cloudItems = [
  { identifier: 'Cloud_Item_1', version: '1.0.0' },
  { identifier: 'Cloud_Item_2', version: '2.1.3' },
  { identifier: 'Cloud_Item_3', version: '3.2.1' }
]

const headers = [
  { title: '', key: 'identifier' },
  { title: '', key: 'actions', width: '60px' }
]

function updateItem(item: any) {
  console.log('Updating item:', item.identifier)
}

function getSourceTypeIcon(sourceType: string) {
  if (sourceType === 'local') {
    return 'mdi-folder-open'
  } else if (sourceType === 'remote') {
    return 'mdi-cloud'
  }
}
</script>

<template>
  <div class="root">
    <div class="full-width">
      <v-btn class="full-width text-none" prepend-icon="mdi-wrench" @click="showMenu = true">
        Manage File List
      </v-btn>
    </div>

    <FileNameTableSelector v-model="selectedValue"></FileNameTableSelector>
  </div>

  <v-dialog v-model="showMenu" width="auto">
    <v-card class="manage-dialog">
      <v-card-text>
        <div class="header-bar">
          <h6 class="text-h6">Manage File List</h6>
          <v-btn icon="mdi-close" flat density="comfortable" @click="showMenu = false"></v-btn>
        </div>

        <div class="btn-row">
          <v-btn class="text-none" prepend-icon="mdi-folder-open">Open Local Dir</v-btn>
          <v-btn class="text-none" prepend-icon="mdi-cloud-download">Check for Updates</v-btn>
        </div>

        <SplitPanel v-model:leftWidth="leftPanelWidth" v-model:rightWidth="rightPanelWidth">
          <template #left>
            <div class="table-container">
              <h6 class="text-h6 ml-2 mt-2 mr-2">Local</h6>
              <v-data-table
                class="local-list"
                :headers="headers"
                :items="items"
                item-value="identifier"
                show-select
                fixed-header
                height="400"
              >
                <template v-slot:item.identifier="{ item }">
                  <span> {{ item.identifier }}</span>
                  <v-icon class="ml-2" :icon="getSourceTypeIcon(item.sourceType)" small></v-icon>
                </template>

                <template v-slot:item.actions="{ item }">
                  <v-btn
                    v-if="item.hasUpdate"
                    size="small"
                    variant="tonal"
                    color="warning"
                    icon="mdi-update"
                    @click.stop="updateItem(item)"
                  >
                  </v-btn>
                </template>

                <template v-slot:bottom>
                  <div class="button-group">
                    <v-btn class="text-none" prepend-icon="mdi-plus">Import</v-btn>
                    <v-btn class="text-none" prepend-icon="mdi-delete">Delete</v-btn>
                    <v-btn class="text-none" prepend-icon="mdi-refresh">Refresh</v-btn>
                  </div>
                </template>
              </v-data-table>
            </div>
          </template>
          <template #right>
            <div class="right-panel-content">
              <div class="cloud-list">
                <h6 class="text-h6 mb-4">Downloadable</h6>
                <v-list density="compact">
                  <v-list-item
                    v-for="item in cloudItems"
                    :key="item.identifier"
                    :title="item.identifier"
                    :subtitle="`Version: ${item.version}`"
                  >
                    <template v-slot:append>
                      <v-btn
                        size="small"
                        variant="tonal"
                        color="primary"
                        icon="mdi-download"
                      ></v-btn>
                    </template>
                  </v-list-item>
                </v-list>
              </div>
            </div>
          </template>
        </SplitPanel>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped lang="scss">
.root {
  display: flex;
  flex-direction: column;
  row-gap: 16px;
}

.full-width {
  width: 100%;
}

.manage-dialog {
  min-width: 650px;
  width: 80vw;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.btn-row {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  margin-bottom: 0.8rem;
}

li {
  list-style: none;
}

.table-container {
  height: 100%;
}

.right-panel-content {
  height: 100%;
}

.table-container {
  flex: 1;
  overflow: hidden;
}

.local-list {
  height: 100%;
  overflow-y: auto;
}

.button-group {
  display: flex;
  gap: 8px;
  padding: 8px;
}

.right-panel {
  flex: 3;
  border-left: 1px solid #ddd;
  padding-left: 16px;
  overflow-y: auto;
  max-height: 400px;

  .cloud-list,
  .update-section {
    padding: 8px;
  }
}
</style>
