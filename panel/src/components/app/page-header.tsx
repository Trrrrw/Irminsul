type PageHeaderProps = {
  title: string;
  description?: string;
};

export function PageHeader({ title, description }: PageHeaderProps) {
  return (
    <header className="space-y-1">
      <h1 className="text-3xl font-semibold tracking-tight text-foreground">{title}</h1>
      {description ? <p className="max-w-2xl text-sm leading-6 text-muted-foreground">{description}</p> : null}
    </header>
  );
}
