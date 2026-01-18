import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'

export const useMarkdown = () => {
    const md: any = new MarkdownIt({
        html: true,
        linkify: true,
        typographer: true,
        highlight: function (str: string, lang: string) {
            if (lang && hljs.getLanguage(lang)) {
                try {
                    return '<pre class="hljs"><code>' +
                        hljs.highlight(str, { language: lang, ignoreIllegals: true }).value +
                        '</code></pre>';
                } catch (__) { }
            }
            return '<pre class="hljs"><code>' + md.utils.escapeHtml(str) + '</code></pre>';
        }
    })

    // Custom rule to handle partial code blocks in streaming
    const defaultRender = md.renderer.rules.fence || function (tokens: any[], idx: number, options: any, env: any, self: any) {
        return self.renderToken(tokens, idx, options);
    };

    md.renderer.rules.fence = function (tokens: any[], idx: number, options: any, env: any, self: any) {
        return defaultRender(tokens, idx, options, env, self);
    };

    const render = (content: string) => {
        if (!content) return ''
        return md.render(content)
    }

    return { render }
}
