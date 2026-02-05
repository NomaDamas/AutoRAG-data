<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useConnectionStore, useDocumentsStore, useUiStore, type DatabaseConfig } from '@/stores'

const connectionStore = useConnectionStore()
const documentsStore = useDocumentsStore()
const uiStore = useUiStore()

const form = ref<DatabaseConfig>({
  host: 'localhost',
  port: 5432,
  database: 'autorag',
  username: 'postgres',
  password: '',
})

// Load saved config if available
watch(
  () => uiStore.isConnectionDialogOpen,
  (isOpen) => {
    if (isOpen && connectionStore.config) {
      form.value = { ...connectionStore.config }
    }
  }
)

async function handleConnect() {
  const success = await connectionStore.connect(form.value)
  if (success) {
    uiStore.closeConnectionDialog()
    await documentsStore.loadDocuments()
  }
}

async function handleDisconnect() {
  await connectionStore.disconnect()
  documentsStore.clearCurrentDocument()
}
</script>

<template>
  <Dialog v-model:open="uiStore.isConnectionDialogOpen">
    <DialogContent class="bg-gray-800 border-gray-700 text-gray-100 sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Database Connection</DialogTitle>
        <DialogDescription class="text-gray-400">
          Connect to your PostgreSQL database containing the AutoRAG-Research schema.
        </DialogDescription>
      </DialogHeader>

      <form class="space-y-4" @submit.prevent="handleConnect">
        <div class="grid gap-4">
          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="host" class="text-right">Host</Label>
            <Input
              id="host"
              v-model="form.host"
              class="col-span-3 bg-gray-700 border-gray-600"
              placeholder="localhost"
            />
          </div>

          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="port" class="text-right">Port</Label>
            <Input
              id="port"
              v-model.number="form.port"
              type="number"
              class="col-span-3 bg-gray-700 border-gray-600"
              placeholder="5432"
            />
          </div>

          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="database" class="text-right">Database</Label>
            <Input
              id="database"
              v-model="form.database"
              class="col-span-3 bg-gray-700 border-gray-600"
              placeholder="autorag"
            />
          </div>

          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="username" class="text-right">Username</Label>
            <Input
              id="username"
              v-model="form.username"
              class="col-span-3 bg-gray-700 border-gray-600"
              placeholder="postgres"
            />
          </div>

          <div class="grid grid-cols-4 items-center gap-4">
            <Label for="password" class="text-right">Password</Label>
            <Input
              id="password"
              v-model="form.password"
              type="password"
              class="col-span-3 bg-gray-700 border-gray-600"
              placeholder="Enter password"
            />
          </div>
        </div>

        <div v-if="connectionStore.connectionError" class="rounded-md bg-red-900/50 p-3 text-sm text-red-300">
          {{ connectionStore.connectionError }}
        </div>

        <DialogFooter class="gap-2">
          <Button
            v-if="connectionStore.isConnected"
            type="button"
            variant="outline"
            class="border-gray-600 hover:bg-gray-700"
            @click="handleDisconnect"
          >
            Disconnect
          </Button>
          <Button
            type="submit"
            :disabled="connectionStore.isConnecting"
            class="bg-blue-600 hover:bg-blue-700"
          >
            <span v-if="connectionStore.isConnecting" class="i-mdi-loading animate-spin mr-2" />
            {{ connectionStore.isConnected ? 'Reconnect' : 'Connect' }}
          </Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>
</template>
