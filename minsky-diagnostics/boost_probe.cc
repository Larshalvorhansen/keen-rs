#include <boost/version.hpp>
#include <boost/filesystem.hpp>
#include <iostream>

int main() {
  std::cout << BOOST_LIB_VERSION << "\n";
  boost::filesystem::path p("test");
  std::cout << p.string() << "\n";
}
