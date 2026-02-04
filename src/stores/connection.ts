import { invoke } from '@tauri-apps/api/core'
import { acceptHMRUpdate, defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface DatabaseConfig {
  host: string
  port: number
  database: string
  username: string
  password: string
}

export const useConnectionStore = defineStore('connection', () => {
  const isConnected = ref(false)
  const isConnecting = ref(false)
  const connectionError = ref<string | null>(null)
  const config = ref<DatabaseConfig | null>(null)

  const canConnect = computed(() => !isConnecting.value && !isConnected.value)

  async function connect(newConfig: DatabaseConfig) {
    isConnecting.value = true
    connectionError.value = null

    try {
      await invoke('connect_database', { config: newConfig })
      config.value = newConfig
      isConnected.value = true
      return true
    } catch (error) {
      connectionError.value = error instanceof Error ? error.message : String(error)
      return false
    } finally {
      isConnecting.value = false
    }
  }

  async function disconnect() {
    try {
      await invoke('disconnect_database')
      isConnected.value = false
      config.value = null
      connectionError.value = null
      return true
    } catch (error) {
      connectionError.value = error instanceof Error ? error.message : String(error)
      return false
    }
  }

  async function testConnection() {
    try {
      const result = await invoke<boolean>('test_connection')
      return result
    } catch (error) {
      connectionError.value = error instanceof Error ? error.message : String(error)
      return false
    }
  }

  async function checkConnectionStatus() {
    try {
      isConnected.value = await invoke<boolean>('is_connected')
    } catch {
      isConnected.value = false
    }
  }

  return {
    isConnected,
    isConnecting,
    connectionError,
    config,
    canConnect,
    connect,
    disconnect,
    testConnection,
    checkConnectionStatus,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useConnectionStore, import.meta.hot))
}
