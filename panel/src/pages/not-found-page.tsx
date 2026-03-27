import { Link } from "react-router-dom";

import { PageHeader } from "@/components/app/page-header";
import { Button } from "@/components/ui/button";

export function NotFoundPage() {
  return (
    <div className="flex flex-col gap-8">
      <PageHeader title="页面不存在" />
      <div>
        <Button asChild>
          <Link to="/">返回概览</Link>
        </Button>
      </div>
    </div>
  );
}
