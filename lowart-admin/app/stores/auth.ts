import { defineStore } from 'pinia'

export const useAuthStore = defineStore('auth', () => {
    const adminKey = ref(import.meta.client ? sessionStorage.getItem('lowart_admin_key') || '' : '')
    const currentUser = ref(import.meta.client ? JSON.parse(sessionStorage.getItem('lowart_current_user') || 'null') : null)
    const isAuthenticated = computed(() => !!adminKey.value)

    const setAuth = (key: string, user: any) => {
        adminKey.value = key
        currentUser.value = user
        if (import.meta.client) {
            sessionStorage.setItem('lowart_admin_key', key)
            sessionStorage.setItem('lowart_current_user', JSON.stringify(user))
        }
    }

    const logout = () => {
        adminKey.value = ''
        currentUser.value = null
        if (import.meta.client) {
            sessionStorage.removeItem('lowart_admin_key')
            sessionStorage.removeItem('lowart_current_user')
        }
        navigateTo('/login')
    }

    return {
        adminKey,
        currentUser,
        isAuthenticated,
        setAuth,
        logout
    }
})
