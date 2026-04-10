# Contribuyendo a Tonet

¡Gracias por tu interés en contribuir a Tonet! Este navegador web disruptivo está construido con Rust y sigue una filosofía minimalista radical: **nunca cargar páginas web que pesen más de 1MB**.

## Filosofía del Proyecto

Tonet desafía la tendencia moderna de webs sobrecargadas:
- **Rendimiento extremo**: Escrito en Rust para máxima velocidad y eficiencia de memoria
- **Minimalismo radical**: Rechaza automáticamente cualquier página que supere 1MB
- **Experiencia limpia**: Sin anuncios, trackers, o bloatware
- **Soberanía del usuario**: Tú controlas qué contenido se carga

## Primeros Pasos

### Compilación Local

1. **Requisitos**: Rust 1.70+ instalado
2. **Clona el repositorio**:
   ```bash
   git clone https://github.com/usetonet/tonet-browser.git
   cd tonet-browser
   ```
3. **Compila el proyecto**:
   ```bash
   cargo build --release
   ```
4. **Ejecuta Tonet**:
   ```bash
   ./target/release/tonet
   ```

### Estructura del Proyecto

- `crates/tonet/`: Núcleo del navegador en Rust
- `web/landing/`: Sitio web de documentación
- `installer/`, `packaging/`, `wix/`: Scripts de empaquetado
- `.github/workflows/`: CI/CD y releases automáticos

## Contributor License Agreement (CLA)

### ¿Por qué necesitamos un CLA?

Para mantener la integridad legal del proyecto y permitir futuras opciones de licenciamiento comercial, requerimos que todos los contribuidores acepten nuestro CLA.

### Proceso del CLA

1. **Primer Pull Request**: Cuando envías tu primer PR, el bot de CLA Assistant te notificará
2. **Firma digital**: Deberás firmar el acuerdo haciendo clic en el enlace proporcionado
3. **Verificación automática**: Una vez firmado, el bot verificará automáticamente futuras contribuciones

### ¿Qué establece el CLA?

Al contribuir, otorgas a Usetonet:
- Licencia perpetua para usar tu código de forma comercial y no comercial
- Derechos para modificar, distribuir y sublicenciar tus contribuciones
- Mantienes la propiedad de tu código, pero nos das permiso para incluirlo en Tonet

**El CLA es obligatorio** para que podamos aceptar tus contribuciones.

## Guías de Contribución

### Reportar Bugs

1. Verifica que no esté ya reportado
2. Incluye versión de Tonet, sistema operativo y pasos para reproducir
3. Describe el comportamiento esperado vs. el actual

### Sugerir Mejoras

1. Explica el problema que resuelve tu sugerencia
2. Propón una solución clara y concisa
3. Considera el impacto en la filosofía minimalista del proyecto

### Enviar Pull Requests

1. Fork del repositorio
2. Crea una rama para tu feature (`git checkout -b feature/amazing-feature`)
3. Commit de cambios (`git commit -m 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## Código de Conducta

Este proyecto sigue un código de conducta profesional. Esperamos que todos los contribuidores:
- Sean respetuosos y constructivos
- Mantengan discusiones técnicas centradas en el código
- Respeten las decisiones de los mantenedores

## Preguntas Frecuentes

**¿Puedo usar Tonet para proyectos comerciales?**
Consulta la licencia PolyForm Noncommercial en el archivo LICENSE.

**¿Qué pasa si mi página web supera 1MB?**
Tonet mostrará un mensaje claro explicando el límite y sugerirá optimizaciones.

**¿Cómo reporto vulnerabilidades de seguridad?**
Contacta directamente a los mantenedores mediante security@usetonet.com.

---

¡Gracias por ayudar a construir un internet más rápido y eficiente!