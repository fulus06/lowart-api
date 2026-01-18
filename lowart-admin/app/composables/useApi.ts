export const useApi = () => {
    const config = useRuntimeConfig()
    const baseUrl = 'http://localhost:8080'
    const adminKey = 'admin_default_key' // Should be dynamic in real use

    const fetchWithAuth = async (url: string, options: any = {}) => {
        return await $fetch(url, {
            baseURL: baseUrl,
            ...options,
            headers: {
                'Authorization': `Bearer ${adminKey}`,
                ...options.headers,
            }
        })
    }

    return {
        // User APIs
        getUsers: () => fetchWithAuth('/admin/users'),
        updateQuota: (payload: { user_id: string, rpm_limit: number, token_quota: number }) => fetchWithAuth('/admin/users/quota', {
            method: 'POST',
            body: payload
        }),

        // Model APIs
        getModels: () => fetchWithAuth('/admin/models'),
        registerMcp: (payload: { name: string, command: string, args: string[] }) => fetchWithAuth('/admin/mcp/register', {
            method: 'POST',
            body: payload
        }),
        unregisterMcp: (name: string) => fetchWithAuth('/admin/mcp/unregister', {
            method: 'POST',
            body: { name }
        }),

        // Stats APIs
        getStats: () => fetchWithAuth('/admin/stats'),

        // Chat APIs
        chat: (payload: any) => {
            return fetchWithAuth('/v1/chat/completions', {
                method: 'POST',
                body: payload
            })
        }
    }
}
