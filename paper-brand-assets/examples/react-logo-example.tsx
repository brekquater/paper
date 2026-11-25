type PaperLogoProps = { theme?: "light" | "dark"; className?: string };

export function PaperLogo({ theme = "light", className }: PaperLogoProps) {
  const src = theme === "dark"
    ? "/brand/logos/paper-logo-horizontal-dark.svg"
    : "/brand/logos/paper-logo-horizontal-light.svg";

  return (
    <img
      className={className}
      src={src}
      alt="Paper — Document Editing Engine by Brekquater"
      width={1240}
      height={220}
      loading="eager"
    />
  );
}
