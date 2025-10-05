/* Mica Futuristic Design System - Shared JavaScript */

// Theme Toggle
function initThemeToggle() {
    const themeToggle = document.getElementById('themeToggle');
    const htmlElement = document.documentElement;
    
    if (!themeToggle) return;
    
    // Load saved theme or default to dark
    const savedTheme = localStorage.getItem('mica-theme') || 'dark';
    htmlElement.setAttribute('data-theme', savedTheme);
    themeToggle.textContent = savedTheme === 'light' ? '🌙' : '☀️';
    
    themeToggle.addEventListener('click', () => {
        const currentTheme = htmlElement.getAttribute('data-theme');
        const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
        
        htmlElement.setAttribute('data-theme', newTheme);
        localStorage.setItem('mica-theme', newTheme);
        themeToggle.textContent = newTheme === 'light' ? '🌙' : '☀️';
    });
}

// Scroll to Top Button
function initScrollTop() {
    const scrollTop = document.getElementById('scrollTop');
    
    if (!scrollTop) return;
    
    window.addEventListener('scroll', () => {
        if (window.pageYOffset > 300) {
            scrollTop.classList.add('visible');
        } else {
            scrollTop.classList.remove('visible');
        }
    });
    
    scrollTop.addEventListener('click', () => {
        window.scrollTo({
            top: 0,
            behavior: 'smooth'
        });
    });
}

// Smooth Scroll for Anchor Links
function initSmoothScroll() {
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            const href = this.getAttribute('href');
            if (href === '#') return;
            
            e.preventDefault();
            const target = document.querySelector(href);
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
}

// Intersection Observer for Scroll Animations
function initScrollAnimations() {
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
            }
        });
    }, observerOptions);
    
    document.querySelectorAll('.animate-on-scroll').forEach(el => {
        observer.observe(el);
    });
}

// Parallax Effect for Mouse Movement
function initParallax() {
    document.addEventListener('mousemove', (e) => {
        const cards = document.querySelectorAll('.card, .feature-card, .quick-link');
        const x = e.clientX / window.innerWidth;
        const y = e.clientY / window.innerHeight;
        
        cards.forEach(card => {
            const rect = card.getBoundingClientRect();
            const cardX = rect.left + rect.width / 2;
            const cardY = rect.top + rect.height / 2;
            
            const distX = (e.clientX - cardX) / 50;
            const distY = (e.clientY - cardY) / 50;
            
            if (card.matches(':hover')) {
                card.style.transform = `translate(${distX}px, ${distY}px) scale(1.02)`;
            }
        });
    });
}

// Copy Code Block
function initCodeCopy() {
    document.querySelectorAll('.code-block, .code-example').forEach(block => {
        const button = document.createElement('button');
        button.className = 'copy-btn';
        button.textContent = '📋 Copy';
        button.style.cssText = `
            position: absolute;
            top: 1rem;
            right: 1rem;
            background: var(--gradient-primary);
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 0.9rem;
            font-weight: 600;
            opacity: 0.7;
            transition: all 0.3s;
            z-index: 10;
        `;
        
        button.addEventListener('mouseover', () => {
            button.style.opacity = '1';
            button.style.transform = 'scale(1.05)';
        });
        
        button.addEventListener('mouseout', () => {
            button.style.opacity = '0.7';
            button.style.transform = 'scale(1)';
        });
        
        button.addEventListener('click', async () => {
            const code = block.textContent;
            try {
                await navigator.clipboard.writeText(code);
                button.textContent = '✅ Copied!';
                setTimeout(() => {
                    button.textContent = '📋 Copy';
                }, 2000);
            } catch (err) {
                button.textContent = '❌ Failed';
                setTimeout(() => {
                    button.textContent = '📋 Copy';
                }, 2000);
            }
        });
        
        block.style.position = 'relative';
        block.appendChild(button);
    });
}

// Initialize all features
document.addEventListener('DOMContentLoaded', () => {
    initThemeToggle();
    initScrollTop();
    initSmoothScroll();
    initScrollAnimations();
    initCodeCopy();
    
    // Optional: Enable parallax on desktop only
    if (window.innerWidth > 1024) {
        initParallax();
    }
});

// Add loading animation
window.addEventListener('load', () => {
    document.body.classList.add('loaded');
});
