export default defineNuxtRouteMiddleware((to, from) => {
    // Skip on server-side as sessionStorage is only available on client
    if (import.meta.server) {
        return
    }

    const authStore = useAuthStore()

    // SSR Hydration fix: If store is empty but sessionStorage has the key, restore it
    if (!authStore.isAuthenticated && import.meta.client) {
        const savedKey = sessionStorage.getItem('lowart_admin_key')
        const savedUser = sessionStorage.getItem('lowart_current_user')
        if (savedKey && savedUser) {
            try {
                authStore.setAuth(savedKey, JSON.parse(savedUser))
            } catch (e) {
                console.error('Failed to restore session:', e)
            }
        }
    }

    // Allow access to login page
    if (to.path === '/login') {
        // If already authenticated, redirect to home
        if (authStore.isAuthenticated) {
            return navigateTo('/')
        }
        return
    }

    // Redirect to login if not authenticated
    if (!authStore.isAuthenticated) {
        return navigateTo('/login')
    }
})
