#include <stdio.h>
int simple_dll_function() {
  printf("call simple_dll_function\n");
  return 1;
}

struct rust_object {
  int a;
  int b;
};

int simple_dll_function_with_param(struct rust_object* p_rust_object) {
  printf("p_rust_object: %p\n", p_rust_object);
  int a = p_rust_object->a;
  printf("a: %d\n", a);

  int b = p_rust_object->b;
  printf("b: %d\n", b);
  printf("simple_dll_function_with_param \n");
  p_rust_object->b = 20;
  return 1;
}