export const useApi = () => {
    const config = useRuntimeConfig()
    const authStore = useAuthStore()
    const baseUrl = 'http://localhost:8080'
    const adminKey = computed(() => authStore.adminKey)

    const getAuthHeaders = () => ({
        'Authorization': `Bearer ${adminKey.value}`,
        'Content-Type': 'application/json'
    })

    const fetchWithAuth = async (url: string, options: any = {}) => {
        return await $fetch(url, {
            baseURL: baseUrl,
            ...options,
            headers: {
                ...getAuthHeaders(),
                ...options.headers,
            }
        })
    }

    return {
        baseUrl,
        getAuthHeaders,
        // Auth API
        login: (api_key: string) => $fetch(`${baseUrl}/admin/login`, {
            method: 'POST',
            body: { api_key }
        }),

        // User APIs
        getUsers: () => fetchWithAuth('/admin/users'),
        createUser: (payload: { username: string, api_key: string, is_admin: boolean }) => fetchWithAuth('/admin/users', {
            method: 'POST',
            body: payload
        }),
        updateUser: (payload: { user_id: string, username: string, api_key: string, status: string }) => fetchWithAuth('/admin/users', {
            method: 'PUT',
            body: payload
        }),
        deleteUser: (user_id: string) => fetchWithAuth('/admin/users', {
            method: 'DELETE',
            body: { user_id }
        }),
        updateQuota: (payload: { user_id: string, rpm_limit: number, token_quota: number }) => fetchWithAuth('/admin/users/quota', {
            method: 'POST',
            body: payload
        }),

        // API Key APIs
        getUserKeys: (user_id: string) => fetchWithAuth(`/admin/users/${user_id}/keys`),
        createKey: (payload: { user_id: string, label: string }) => fetchWithAuth('/admin/keys', {
            method: 'POST',
            body: payload
        }),
        resetKey: (key_id: number) => fetchWithAuth('/admin/keys/reset', {
            method: 'POST',
            body: { key_id }
        }),
        deleteKey: (key_id: number) => fetchWithAuth('/admin/keys/delete', {
            method: 'POST',
            body: { key_id }
        }),

        // Model APIs
        getModels: () => fetchWithAuth('/admin/models'),
        createModel: (payload: { title: string, model_id: string, api_key: string, base_url: string, vendor_type: string, cost_per_1k_tokens: number, is_active: boolean }) => fetchWithAuth('/admin/models', {
            method: 'POST',
            body: payload
        }),
        updateModel: (payload: { id: string, title: string, model_id: string, api_key: string, base_url: string, vendor_type: string, cost_per_1k_tokens: number, is_active: boolean }) => fetchWithAuth('/admin/models', {
            method: 'PUT',
            body: payload
        }),
        deleteModel: (id: string) => fetchWithAuth('/admin/models', {
            method: 'DELETE',
            body: { id }
        }),
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
