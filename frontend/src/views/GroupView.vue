<template>
  <div class="flex max-h-screen p-0 m-0">
    <div class="w-screen" style="position: absolute; top: 0; left: 0; z-index: 9;">
      <div v-if="santaStore.isLoading" class="text-center w-full">
        Loading...
      </div>

      <template v-else>
        <Card class="mt-4" style="width: 350px; margin-left: calc(50% - 174px)">
          <template #content>
            <div class="mb-4">
              <h1 class="text-2xl text-center mt-0 font-bold">{{ santaStore.currentGroup?.name }}</h1>
              <p>Group Code: {{ groupId }}</p>
            </div>

            <div class="mb-4">
              <h2 class="text-xl font-semibold mb-3">Members:</h2>
              <div 
                v-for="member in santaStore.currentGroup?.members"
                :key="member.name"
                class="flex flex-row gap-2 border-1 border-solid p-2 mb-2" 
                style="border-radius: 8px;"
              >
                <div class="p-1">{{ member.name }}</div>
                <div>
                  <Tag v-if="member.isCreator" severity="info" value="Creator" class="mr-2"></Tag>
                  <Tag v-if="member.name === santaStore.currentUser" severity="success" value="You"></Tag>
                </div>
              </div>
            </div>

            <div v-if="santaStore.isCreator && !santaStore.currentGroup?.isGenerated" class="mb-4">
              <Button
                @click="handleGenerate"
                :disabled="(santaStore.currentGroup?.members.length ?? 0) % 2 !== 0"
                class="w-full"
                severity="success"
              >
                Generate Secret Santas
              </Button>
              <p 
                v-if="(santaStore.currentGroup?.members.length ?? 0) % 2 !== 0" 
                class="text-sm mt-2" 
                style="color: #ff6b6b;"
              >
                Need an even number of members to generate pairs
              </p>
            </div>

            <div v-if="santaStore.secretSantaFor" class="mb-4 p-2" style="background-color: rgba(34, 197, 94, 0.1); border-radius: 8px;">
              <h3 class="font-semibold mt-2 text-center">Your Secret Santa Assignment:</h3>
              <p class="text-lg mt-2 text-center mt-4"><strong>{{ santaStore.secretSantaFor }}</strong></p>
            </div>

            <div class="flex justify-content-between">
              <Button
                @click="router.push('/')"
                severity="secondary"
              >
                Back to Home
              </Button>
              <Button
                @click="copyGroupLink"
                severity="info"
              >
                Copy Invite Link
              </Button>
            </div>
            
          </template>
        </Card>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, getCurrentInstance } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSantaStore } from '@/stores/santa'

const instance = getCurrentInstance();

const router = useRouter()
const route = useRoute()
const santaStore = useSantaStore()
const groupId = route.params.id as string

onMounted(async () => {
  console.log('groupId', groupId)
  await santaStore.loadGroup(groupId);

  const isMember = santaStore.currentGroup?.members.some(g => g.userId === santaStore.currentUserId)
  if (!isMember) {
    const join = confirm('You are not a member of this group. Would you like to join?');
    if (join) {
      await santaStore.joinGroup(groupId)
      await santaStore.loadGroup(groupId)
    } else {
      router.push('/')
      return
    }
  }
  santaStore.connectWebSocket(groupId)
})

onBeforeUnmount(() => {
  santaStore.disconnectWebSocket()
})

async function handleGenerate() {
  await santaStore.generatePairs()
  if (instance && instance?.proxy) {
    instance?.proxy.$forceUpdate();
  }
}

function copyGroupLink() {
  const code = `${window.location.origin}/group/${groupId}`
  navigator.clipboard.writeText(code)
}
</script>
