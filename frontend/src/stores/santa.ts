import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

interface Member {
  userId: string
  name: string
  isCreator: boolean
}

interface Group {
  id: string
  name: string
  creator: string
  members: Member[]
  isGenerated: boolean
}

interface UserGroup {
  id: string
  name: string
  isGenerated: boolean
  isCreator: boolean
}

export const useSantaStore = defineStore('santa', () => {
  const currentGroup = ref<Group | null>(null)
  const currentUser = ref<string>('')
  const currentUserId = ref<string>('')
  const secretSantaFor = ref<string>('')
  const isLoading = ref(false)
  const ws = ref<WebSocket | null>(null)
  const userGroups = ref<UserGroup[]>([])

  const isCreator = computed(() => {
    return currentGroup.value?.creator === currentUser.value
  })

  function generateUserId(): string {
    return crypto.randomUUID()
  }

  function getSiteUser() {
    const stored = localStorage.getItem('site_user')
    if (stored) {
      const data = JSON.parse(stored)
      currentUser.value = data.name
      currentUserId.value = data.userId
      return data
    }
    return null
  }

  function setSiteUser(name: string) {
    const userId = generateUserId()
    currentUser.value = name
    currentUserId.value = userId
    localStorage.setItem('site_user', JSON.stringify({ name, userId }))
  }

  function getAssignment(groupId: string) {
    return localStorage.getItem(`santa_${groupId}_${currentUserId.value}`) || ''
  }

  function setAssignment(groupId: string, receiver: string) {
    localStorage.setItem(`santa_${groupId}_${currentUserId.value}`, receiver)
    secretSantaFor.value = receiver
  }

  function clearAssignment() {
    secretSantaFor.value = ''
  }

  async function createGroup(name: string) {
    const response = await fetch(`${import.meta.env.VITE_API_URL}/api/create-group`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        name,
        creator: currentUser.value,
        userId: currentUserId.value
      })
    })
    if (!response.ok) {
      throw new Error(await response.text())
    }
    const data = await response.json()
    console.log(data);
    return data.id
  }

  async function joinGroup(groupId: string) {
    const response = await fetch(`${import.meta.env.VITE_API_URL}/api/join-group`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        groupId,
        name: currentUser.value,
        userId: currentUserId.value
      })
    })
    if (!response.ok) {
      throw new Error(await response.text())
    }
  }

  async function loadGroup(groupId: string) {
    isLoading.value = true
    try {
      const response = await fetch(`${import.meta.env.VITE_API_URL}/api/group?id=${groupId}`)
      if (!response.ok) {
        throw new Error('Failed to load group')
      }
      currentGroup.value = await response.json()
      secretSantaFor.value = getAssignment(groupId)
    } finally {
      isLoading.value = false
    }
  }

  async function loadUserGroups() {
    if (!currentUserId.value) return
    const response = await fetch(`${import.meta.env.VITE_API_URL}/api/user-groups?userId=${currentUserId.value}`)
    if (response.ok) {
      userGroups.value = await response.json()
    }
  }

  function connectWebSocket(groupId: string) {
    clearAssignment()
    disconnectWebSocket()

    ws.value = new WebSocket(
      `${import.meta.env.VITE_WS_URL}/ws?id=${groupId}&userId=${currentUserId.value}`
    )

    ws.value.onmessage = (event) => {
      const message = JSON.parse(event.data)

      switch (message.type) {
        case 'group_update':
          currentGroup.value = message.data
          break
        case 'assignment':
          setAssignment(groupId, message.data.receiver)
          break
        case 'assignments_generated':
          if (message.data.byUserId[currentUserId.value]) {
            setAssignment(groupId, message.data.byUserId[currentUserId.value])
          }
          break
      }
    }

    ws.value.onerror = (error) => {
      console.error('WebSocket error:', error)
    }

    ws.value.onclose = () => {
      setTimeout(() => connectWebSocket(groupId), 1000)
    }
  }

  function disconnectWebSocket() {
    if (ws.value) {
      ws.value.onclose = null // Prevent reconnect
      ws.value.close()
      ws.value = null
    }
  }

  async function generatePairs() {
    if (!currentGroup.value) return

    const response = await fetch(`${import.meta.env.VITE_API_URL}/api/generate-pairs?id=${currentGroup.value.id}`, {
      method: 'POST'
    })
    if (!response.ok) {
      throw new Error(await response.text())
    }
  }

  return {
    currentGroup,
    currentUser,
    currentUserId,
    secretSantaFor,
    isLoading,
    isCreator,
    userGroups,
    getSiteUser,
    setSiteUser,
    createGroup,
    joinGroup,
    loadGroup,
    loadUserGroups,
    connectWebSocket,
    disconnectWebSocket,
    generatePairs,
    clearAssignment
  }
})
