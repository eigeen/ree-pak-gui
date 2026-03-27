# Data Tables with TanStack Table

## Installation

```bash
npx shadcn-vue@latest add data-table

npm install @tanstack/vue-table
```

## Basic Setup

```vue
<script setup lang="ts">
import { DataTable } from "@/components/ui/data-table";
import { h } from "vue";

const columns = [
  {
    accessorKey: "id",
    header: "ID",
  },
  {
    accessorKey: "name",
    header: "Name",
  },
  {
    accessorKey: "email",
    header: "Email",
  },
];

const data = [
  { id: 1, name: "John Doe", email: "john@example.com" },
  { id: 2, name: "Jane Smith", email: "jane@example.com" },
];
</script>

<template>
  <DataTable :columns="columns" :data="data" />
</template>
```

**Features**: Sorting, filtering, pagination, row selection, column visibility, expandable rows
