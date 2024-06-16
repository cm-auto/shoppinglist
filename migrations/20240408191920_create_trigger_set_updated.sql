create function trigger_set_updated()
returns trigger as $$
begin
  new.updated = NOW();
  return new;
end;
$$ language plpgsql;