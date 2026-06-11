import { Directive, ElementRef, OnInit, NgZone } from '@angular/core';

@Directive({
  selector: '[appRipple]',
  standalone: true,
})
export class RippleDirective implements OnInit {
  constructor(private el: ElementRef<HTMLElement>, private ngZone: NgZone) {}

  ngOnInit(): void {
    const el = this.el.nativeElement;
    el.style.position = 'relative';
    el.style.overflow = 'hidden';

    this.ngZone.runOutsideAngular(() => {
      el.addEventListener('click', (e: MouseEvent) => {
        const rect = el.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const size = Math.max(rect.width, rect.height) * 1.5;

        const ripple = document.createElement('span');
        ripple.style.cssText = `
          position: absolute; top: ${y - size / 2}px; left: ${x - size / 2}px;
          width: ${size}px; height: ${size}px; border-radius: 50%;
          background: rgba(14, 165, 233, 0.15); pointer-events: none;
          animation: rippleAnim 0.6s ease-out;
        `;

        el.appendChild(ripple);
        setTimeout(() => ripple.remove(), 600);
      });
    });
  }
}
