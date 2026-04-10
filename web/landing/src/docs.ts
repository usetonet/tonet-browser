import "./styles/global.css";

document.querySelectorAll<HTMLButtonElement>(".copy-btn").forEach((btn) => {
  btn.addEventListener("click", async () => {
    const id = btn.dataset.copy;
    if (!id) return;
    const el = document.getElementById(id);
    if (!el) return;
    const text = el.innerText;
    try {
      await navigator.clipboard.writeText(text);
      const prev = btn.textContent;
      btn.textContent = "¡Copiado!";
      setTimeout(() => {
        btn.textContent = prev;
      }, 1600);
    } catch {
      btn.textContent = "Error";
      setTimeout(() => {
        btn.textContent = "Copiar";
      }, 1600);
    }
  });
});
