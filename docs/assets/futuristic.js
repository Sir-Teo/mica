/* ⚡ MICA ULTRA-FUTURISTIC DESIGN SYSTEM - INTERACTIVE JAVASCRIPT ⚡ */

// Enhanced Theme Toggle with Smooth Transition
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
        
        // Add transition class
        document.body.style.transition = 'background-color 0.5s ease, color 0.5s ease';
        
        htmlElement.setAttribute('data-theme', newTheme);
        localStorage.setItem('mica-theme', newTheme);
        themeToggle.textContent = newTheme === 'light' ? '🌙' : '☀️';
        
        // Create ripple effect
        createRipple(themeToggle);
        
        // Remove transition after animation
        setTimeout(() => {
            document.body.style.transition = '';
        }, 500);
    });
    
    // Add pulse animation on hover
    themeToggle.addEventListener('mouseenter', () => {
        themeToggle.style.animation = 'pulse 0.5s ease';
    });
    
    themeToggle.addEventListener('animationend', () => {
        themeToggle.style.animation = '';
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

// Advanced 3D Parallax Effect for Mouse Movement
function initParallax() {
    let ticking = false;
    
    document.addEventListener('mousemove', (e) => {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                const cards = document.querySelectorAll('.card, .feature-card, .quick-link, .module-card');
                const x = (e.clientX / window.innerWidth - 0.5) * 2;
                const y = (e.clientY / window.innerHeight - 0.5) * 2;
                
                cards.forEach((card, index) => {
                    if (!card.matches(':hover')) return;
                    
                    const rect = card.getBoundingClientRect();
                    const cardCenterX = rect.left + rect.width / 2;
                    const cardCenterY = rect.top + rect.height / 2;
                    
                    const deltaX = (e.clientX - cardCenterX) / rect.width;
                    const deltaY = (e.clientY - cardCenterY) / rect.height;
                    
                    const rotateX = deltaY * 10;
                    const rotateY = deltaX * 10;
                    const translateZ = 20;
                    
                    card.style.transform = `
                        perspective(1000px)
                        rotateX(${-rotateX}deg)
                        rotateY(${rotateY}deg)
                        translateZ(${translateZ}px)
                        scale(1.05)
                    `;
                });
                
                ticking = false;
            });
            
            ticking = true;
        }
    });
    
    // Reset transform when mouse leaves
    document.querySelectorAll('.card, .feature-card, .quick-link, .module-card').forEach(card => {
        card.addEventListener('mouseleave', () => {
            card.style.transform = '';
        });
    });
}

// Create Ripple Effect
function createRipple(element) {
    const ripple = document.createElement('span');
    const rect = element.getBoundingClientRect();
    const size = Math.max(rect.width, rect.height);
    
    ripple.style.cssText = `
        position: absolute;
        border-radius: 50%;
        background: rgba(139, 92, 246, 0.4);
        width: ${size}px;
        height: ${size}px;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%) scale(0);
        animation: ripple 0.6s ease-out;
        pointer-events: none;
    `;
    
    element.style.position = 'relative';
    element.style.overflow = 'hidden';
    element.appendChild(ripple);
    
    setTimeout(() => ripple.remove(), 600);
}

// Add ripple animation to CSS dynamically
const style = document.createElement('style');
style.textContent = `
    @keyframes ripple {
        to {
            transform: translate(-50%, -50%) scale(2);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);

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

// Particle Cursor Trail Effect
function initCursorTrail() {
    if (window.innerWidth < 768) return; // Skip on mobile
    
    const particles = [];
    const particleCount = 15;
    
    document.addEventListener('mousemove', (e) => {
        if (Math.random() > 0.8) { // Only create particle 20% of the time
            const particle = document.createElement('div');
            particle.className = 'cursor-particle';
            particle.style.cssText = `
                position: fixed;
                width: 4px;
                height: 4px;
                background: linear-gradient(135deg, #8b5cf6, #ec4899);
                border-radius: 50%;
                pointer-events: none;
                z-index: 9999;
                left: ${e.clientX}px;
                top: ${e.clientY}px;
                opacity: 1;
                animation: particleFade 1s ease-out forwards;
                box-shadow: 0 0 10px rgba(139, 92, 246, 0.6);
            `;
            
            document.body.appendChild(particle);
            
            setTimeout(() => particle.remove(), 1000);
        }
    });
    
    // Add particle animation
    const particleStyle = document.createElement('style');
    particleStyle.textContent = `
        @keyframes particleFade {
            to {
                transform: translateY(-30px) scale(0);
                opacity: 0;
            }
        }
    `;
    document.head.appendChild(particleStyle);
}

// Floating Particles Background
function initFloatingParticles() {
    if (window.innerWidth < 768) return; // Skip on mobile
    
    const particleContainer = document.createElement('div');
    particleContainer.id = 'particle-container';
    particleContainer.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
        z-index: 1;
        overflow: hidden;
    `;
    document.body.appendChild(particleContainer);
    
    // Create particles
    for (let i = 0; i < 30; i++) {
        const particle = document.createElement('div');
        const size = Math.random() * 3 + 1;
        const duration = Math.random() * 20 + 10;
        const delay = Math.random() * 5;
        
        particle.style.cssText = `
            position: absolute;
            width: ${size}px;
            height: ${size}px;
            background: radial-gradient(circle, rgba(139, 92, 246, 0.8), transparent);
            border-radius: 50%;
            top: ${Math.random() * 100}%;
            left: ${Math.random() * 100}%;
            animation: float ${duration}s ${delay}s infinite ease-in-out;
            opacity: 0.6;
        `;
        
        particleContainer.appendChild(particle);
    }
    
    // Add float animation
    const floatStyle = document.createElement('style');
    floatStyle.textContent = `
        @keyframes float {
            0%, 100% {
                transform: translate(0, 0) scale(1);
                opacity: 0.6;
            }
            25% {
                transform: translate(20px, -30px) scale(1.2);
                opacity: 0.8;
            }
            50% {
                transform: translate(-15px, -60px) scale(0.8);
                opacity: 0.4;
            }
            75% {
                transform: translate(-30px, -30px) scale(1.1);
                opacity: 0.7;
            }
        }
    `;
    document.head.appendChild(floatStyle);
}

// Add ripple effect to all buttons
function initButtonRipples() {
    document.querySelectorAll('.btn, .btn-primary, .btn-secondary').forEach(button => {
        button.addEventListener('click', function(e) {
            createRipple(this);
        });
    });
}

// Performance Monitoring (show FPS in dev mode)
function initPerformanceMonitor() {
    if (window.location.search.includes('debug=true')) {
        let lastTime = performance.now();
        let fps = 0;
        
        const fpsDisplay = document.createElement('div');
        fpsDisplay.style.cssText = `
            position: fixed;
            top: 10px;
            left: 10px;
            background: rgba(0, 0, 0, 0.8);
            color: #00ff00;
            padding: 5px 10px;
            font-family: monospace;
            font-size: 12px;
            border-radius: 4px;
            z-index: 10000;
        `;
        document.body.appendChild(fpsDisplay);
        
        function updateFPS() {
            const currentTime = performance.now();
            fps = Math.round(1000 / (currentTime - lastTime));
            lastTime = currentTime;
            fpsDisplay.textContent = `FPS: ${fps}`;
            requestAnimationFrame(updateFPS);
        }
        
        updateFPS();
    }
}

// Initialize all features
document.addEventListener('DOMContentLoaded', () => {
    initThemeToggle();
    initScrollTop();
    initSmoothScroll();
    initScrollAnimations();
    initCodeCopy();
    initButtonRipples();
    initPerformanceMonitor();
    
    // Optional: Enable advanced effects on desktop only
    if (window.innerWidth > 1024) {
        initParallax();
        initCursorTrail();
        initFloatingParticles();
    }
    
    // Add custom cursor effect
    document.body.style.cursor = 'default';
});

// Add loading animation
window.addEventListener('load', () => {
    document.body.classList.add('loaded');
    
    // Add a welcome animation
    console.log('%c⚡ MICA - Next-Gen Systems Language ⚡', 
        'font-size: 20px; font-weight: bold; background: linear-gradient(135deg, #8b5cf6, #ec4899); -webkit-background-clip: text; -webkit-text-fill-color: transparent;');
    console.log('%cWelcome to the future of systems programming!', 
        'font-size: 14px; color: #8b5cf6;');
});
