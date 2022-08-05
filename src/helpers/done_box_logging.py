from helpers.color import *
import os
import sys


class log_box:
  def __init__(self, text, status='Pending', color='MAGENTA', silent=False):
    self.text = text
    self.status = status
    self.color = color
    self.silent = silent
    if not silent:
      print("[" + fg[color] + self.status + style["RESET_ALL"] + "] " + str(self.text), end='\r')
  
  def set(self, status, color='MAGENTA'):
    self.status = status
    self.color = color
    self.update()
  
  def update(self):
    if not self.silent:
      width, _ = os.get_terminal_size()
      sys.stdout.write('\r' + ' ' * width)
      sys.stdout.flush()
      print("[" + fg[self.color] + self.status + style["RESET_ALL"] + "] " + str(self.text), end='\r')


  def end(self, status='DONE', color='GREEN'):
    self.status = status
    self.color = color
    if not self.silent:
      width, _ = os.get_terminal_size()
      sys.stdout.write(' ' * width + '\r')
      sys.stdout.flush()
      print("[" + fg[color] + self.status + style["RESET_ALL"] + "] " + str(self.text))
  
 