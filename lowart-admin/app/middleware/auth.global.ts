export default defineNuxtRouteMiddleware((to, from) => {
    const authStore = useAuthStore()

    // Allow access to login page
    if (to.path === '/login') {
        return
    }

    // Redirect to login if not authenticated
    if (!authStore.isAuthenticated) {
        return navigateTo('/login')
    }
})
